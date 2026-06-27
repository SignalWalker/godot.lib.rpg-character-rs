use std::collections::VecDeque;

use godot::classes::{
    AnimatedSprite2D, Node2D,
    class_macros::private::virtuals::{Xrvrs::Gd, ZipReader::Vector2},
};

use crate::{RpgDirection, avatar::CharacterSprite2d};

// #[derive(Debug, thiserror::Error)]
// enum FollowerError {
//     #[error("expected AnimatedSprite2D or Node2D, found {0}")]
//     UnrecognizedType(Gd<Node2D>),
// }

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct FollowerFrame {
    pub position: Vector2,
    pub facing: RpgDirection,
}

enum FollowerNode {
    AnimatedSprite2D(CharacterSprite2d),
    Node2d(Gd<Node2D>),
}

impl std::fmt::Display for FollowerNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FollowerNode::AnimatedSprite2D(sprite) => sprite.sprite.fmt(f),
            FollowerNode::Node2d(n) => n.fmt(f),
        }
    }
}

impl FollowerNode {
    fn from_gd(node: Gd<Node2D>) -> Self {
        match node.try_cast::<AnimatedSprite2D>() {
            Ok(sprite) => {
                Self::AnimatedSprite2D(CharacterSprite2d::new(sprite, RpgDirection::South))
            }
            Err(n) => Self::Node2d(n),
        }
    }

    fn apply_frame(&mut self, pos: FollowerFrame) {
        match self {
            FollowerNode::AnimatedSprite2D(sprite) => {
                sprite.set_dir(pos.facing);
                sprite.ensure_playing();
                sprite.sprite.set_global_position(pos.position);
            }
            FollowerNode::Node2d(n) => n.set_global_position(pos.position),
        }
    }

    fn stop(&mut self) {
        if let Self::AnimatedSprite2D(c) = self {
            c.ensure_stopped()
        }
    }
}

pub(super) struct Follower {
    /// the node this follower controls
    node: FollowerNode,
    /// the number of frames until this starts following
    delay_frames: usize,
    /// the position queue
    frames: VecDeque<FollowerFrame>,
    /// the current frame
    current_frame: FollowerFrame,
}

impl std::fmt::Display for Follower {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.node.fmt(f)
    }
}

impl Follower {
    pub(super) fn from_gd(
        node: Gd<Node2D>,
        delay_frames: usize,
        start_frame: FollowerFrame,
    ) -> Self {
        let mut res = Self {
            node: FollowerNode::from_gd(node),
            delay_frames,
            frames: VecDeque::with_capacity(delay_frames + 1),
            current_frame: start_frame,
        };
        res.node.apply_frame(start_frame);
        res
    }

    fn apply_frame(&mut self, frame: FollowerFrame) {
        self.current_frame = frame;
        self.node.apply_frame(frame);
    }

    fn push_frame(&mut self, frame: FollowerFrame) -> Option<FollowerFrame> {
        self.frames.push_front(frame);
        if self.frames.len() >= self.delay_frames {
            let next = self.frames.pop_back().unwrap();
            self.apply_frame(next);
            Some(next)
        } else {
            None
        }
    }

    pub(super) fn stop(&mut self) {
        self.node.stop();
    }
}

pub(super) struct FollowerSet {
    followers: Vec<Follower>,
}

impl Default for FollowerSet {
    fn default() -> Self {
        Self::new()
    }
}

impl FollowerSet {
    pub(super) const fn new() -> Self {
        Self {
            followers: Vec::new(),
        }
    }

    pub(super) fn push_follower(
        &mut self,
        follower: Gd<Node2D>,
        delay_frames: usize,
        initial_distance: f32,
        leader_frame: FollowerFrame,
    ) {
        fn follower_with_next_frame(
            follower: Gd<Node2D>,
            delay_frames: usize,
            next: FollowerFrame,
            dist: f32,
        ) -> Follower {
            let mut res = Follower::from_gd(
                follower,
                delay_frames,
                FollowerFrame {
                    position: next.position - next.facing.to_vector() * dist,
                    facing: next.facing,
                },
            );
            res.push_frame(next);
            res
        }
        let res = if let Some(back_frame) = self.followers.last().map(|f| f.current_frame) {
            follower_with_next_frame(follower, delay_frames, back_frame, initial_distance)
        } else {
            follower_with_next_frame(follower, delay_frames, leader_frame, initial_distance)
        };
        self.followers.push(res)
    }

    pub(super) fn push_frame(&mut self, pos: Vector2, dir: RpgDirection) {
        // if pushing a frame to a follower overflows its frame buffer and causes it to pop the back
        // frame, the follower applies that back frame to itsel and returns that frame.
        //
        // So, here, we:
        // 1. push a frame
        // 2. if that caused a frame to pop:
        //   3. repeat with the popped frame and the next follower
        let mut frame = FollowerFrame {
            position: pos,
            facing: dir,
        };
        let mut iter = self.followers.iter_mut();
        while let Some(next_frame) = iter.next().and_then(|follower| follower.push_frame(frame)) {
            frame = next_frame;
        }
    }

    pub(super) fn stop(&mut self) {
        for follower in &mut self.followers {
            follower.stop();
        }
    }
}
