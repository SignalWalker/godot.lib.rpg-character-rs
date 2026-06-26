use godot::classes::{Node, class_macros::private::virtuals::Xrvrs::Gd};

use crate::RpgCharacter2d;

pub fn find_first_avatar(root: Gd<Node>) -> Option<Gd<RpgCharacter2d>> {
    let mut queue = vec![root];
    while let Some(node) = queue.pop() {
        match node.try_cast::<RpgCharacter2d>() {
            Ok(av) => return Some(av),
            Err(node) => {
                queue.extend(node.get_children().iter_shared());
            }
        }
    }
    None
}
