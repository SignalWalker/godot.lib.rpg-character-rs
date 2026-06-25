use std::collections::VecDeque;

use godot::classes::{
    AnimatedSprite2D, Node2D,
    class_macros::private::virtuals::{Xrvrs::Gd, ZipReader::Vector2},
};

use crate::{RpgDirection, avatar::CharacterSprite2D};

// #[derive(Debug, thiserror::Error)]
// enum FollowerError {
//     #[error("expected AnimatedSprite2D or Node2D, found {0}")]
//     UnrecognizedType(Gd<Node2D>),
// }

#[derive(Debug, Clone, Copy, PartialEq)]
struct FollowerFrame {
    position: Vector2,
    facing: RpgDirection,
}

enum FollowerNode {
    AnimatedSprite2D(CharacterSprite2D),
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
                Self::AnimatedSprite2D(CharacterSprite2D::new(sprite, RpgDirection::South))
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
}

impl std::fmt::Display for Follower {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.node.fmt(f)
    }
}

impl Follower {
    pub(super) fn from_gd(node: Gd<Node2D>, delay_frames: usize) -> Self {
        Self {
            node: FollowerNode::from_gd(node),
            delay_frames,
            frames: VecDeque::with_capacity(delay_frames + 1),
        }
    }

    fn next_frame(&self) -> Option<&FollowerFrame> {
        self.frames.back()
    }

    fn push_frame(&mut self, frame: FollowerFrame) -> Option<FollowerFrame> {
        self.frames.push_front(frame);
        if self.frames.len() >= self.delay_frames {
            let next = self.frames.pop_back().unwrap();
            self.node.apply_frame(next);
            Some(next)
        } else {
            None
        }
    }

    pub(super) fn stop(&mut self) {
        self.node.stop();
    }

    pub(super) fn reset_to(&mut self, position: Vector2, facing: RpgDirection) {
        self.frames.clear();
        self.node.apply_frame(FollowerFrame { position, facing });
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
        leader_pos: Vector2,
        leader_facing: RpgDirection,
    ) {
        let mut res = Follower::from_gd(follower, delay_frames);
        if let Some(back) = self.followers.last().and_then(Follower::next_frame) {
            res.reset_to(back.position, back.facing);
        } else {
            res.reset_to(leader_pos, leader_facing);
        }
        self.followers.push(res)
    }

    pub(super) fn push_frame(&mut self, pos: Vector2, dir: RpgDirection) {
        // when we push a frame to a follower, if it popped a frame, it returns that frame.
        // So, here, we:
        // 1. push a frame
        // 2. if that caused a frame to pop:
        //   3. repeat with the new frame and the next follower
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
