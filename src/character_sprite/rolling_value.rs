use godot::{
    meta::{FromGodot, GodotConvert, ToGodot},
    register::property::{Export, Var},
};

#[derive(Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RollingValue<T> {
    /// The current value
    pub current: T,
    /// The previous value
    pub prev: T,
}

impl<T> RollingValue<T> {
    pub const fn new(prev: T, current: T) -> Self {
        Self { current, prev }
    }

    pub const fn new_with(both: T) -> Self
    where
        T: Copy,
    {
        Self {
            current: both,
            prev: both,
        }
    }

    pub const fn push(&mut self, new: T) -> T {
        std::mem::swap(&mut self.current, &mut self.prev);
        std::mem::replace(&mut self.current, new)
    }
}

impl<T: GodotConvert> GodotConvert for RollingValue<T> {
    type Via = T::Via;

    fn godot_shape() -> godot::meta::shape::GodotShape {
        T::godot_shape()
    }
}

impl<T: Var + FromGodot + Copy> Var for RollingValue<T> {
    type PubType = T;

    fn var_get(field: &Self) -> Self::Via {
        T::var_get(&field.current)
    }

    fn var_set(field: &mut Self, value: Self::Via) {
        field.push(T::from_godot(value));
    }

    fn var_pub_get(field: &Self) -> Self::PubType {
        field.current
    }

    fn var_pub_set(field: &mut Self, value: Self::PubType) {
        field.push(value);
    }
}

impl<T> Export for RollingValue<T> where RollingValue<T>: Var {}

impl<T: FromGodot + Copy> FromGodot for RollingValue<T> {
    fn try_from_godot(via: Self::Via) -> Result<Self, godot::prelude::ConvertError> {
        T::try_from_godot(via).map(Self::new_with)
    }
}

impl<T: ToGodot + Copy> ToGodot for RollingValue<T> {
    type Pass = T::Pass;

    fn to_godot(&self) -> godot::meta::ToArg<'_, Self::Via, Self::Pass> {
        self.current.to_godot()
    }
}
