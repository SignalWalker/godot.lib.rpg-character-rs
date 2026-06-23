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

    fn set_position(&mut self, prev: Vector2, pos: Vector2) {
        match self {
            FollowerNode::AnimatedSprite2D(sprite) => {
                let dir = RpgDirection::from_vec(pos - prev);
                sprite.set_dir(dir);
                sprite.ensure_playing();
                sprite.sprite.set_global_position(pos);
            }
            FollowerNode::Node2d(n) => n.set_global_position(pos),
        }
    }

    fn stop(&mut self) {
        if let Self::AnimatedSprite2D(c) = self {
            c.ensure_stopped()
        }
    }

    fn current_position(&self) -> Vector2 {
        match self {
            Self::AnimatedSprite2D(c) => c.sprite.get_global_position(),
            Self::Node2d(n) => n.get_global_position(),
        }
    }
}

pub(super) struct Follower {
    node: FollowerNode,
    /// the number of frames until this starts following
    delay_frames: usize,
    /// the position queue
    positions: VecDeque<Vector2>,
    /// the previous position
    prev_pos: Vector2,
}

impl std::fmt::Display for Follower {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.node.fmt(f)
    }
}

impl Follower {
    pub(super) fn from_gd(node: Gd<Node2D>, delay_frames: usize) -> Self {
        Self {
            prev_pos: node.get_global_position(),
            node: FollowerNode::from_gd(node),
            delay_frames,
            positions: VecDeque::with_capacity(delay_frames + 1),
        }
    }

    pub(super) fn push_position(&mut self, pos: Vector2) -> Option<Vector2> {
        self.positions.push_front(pos);
        if self.positions.len() >= self.delay_frames {
            let next = self.positions.pop_back().unwrap();
            self.set_position(next);
            Some(next)
        } else {
            None
        }
    }

    pub(super) fn stop(&mut self) {
        self.node.stop();
    }

    fn set_position(&mut self, pos: Vector2) {
        self.node.set_position(self.prev_pos, pos);
        self.prev_pos = pos;
    }

    pub(super) fn reset_to(&mut self, pos: Vector2) {
        self.positions.clear();
        self.set_position(pos);
        self.node.stop();
    }

    pub(super) fn current_position(&self) -> Vector2 {
        self.node.current_position()
    }
}

pub(super) struct FollowerSet {
    delay_frames: usize,
    followers: Vec<Follower>,
}

impl Default for FollowerSet {
    fn default() -> Self {
        Self::new(12)
    }
}

impl FollowerSet {
    pub(super) fn new(delay_frames: usize) -> Self {
        Self {
            delay_frames,
            followers: Vec::new(),
        }
    }

    pub(super) fn push_follower(&mut self, target_pos: Vector2, follower: Gd<Node2D>) {
        let mut res = Follower::from_gd(follower, self.delay_frames);
        if let Some(back) = self.followers.last() {
            res.reset_to(back.current_position());
        } else {
            res.reset_to(target_pos);
        }
        self.followers.push(res)
    }

    pub(super) fn push_position(&mut self, mut pos: Vector2) {
        let mut iter = self.followers.iter_mut();
        while let Some(next_pos) = iter.next().and_then(|follower| follower.push_position(pos)) {
            pos = next_pos;
        }
    }

    pub(super) fn stop(&mut self) {
        for follower in &mut self.followers {
            follower.stop();
        }
    }
}
