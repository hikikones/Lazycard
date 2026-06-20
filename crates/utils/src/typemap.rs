use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

pub struct TypeMap {
    map: HashMap<TypeId, Box<dyn Any>>,
}

impl TypeMap {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn insert<T: Any>(&mut self, value: T) {
        self.map.insert(TypeId::of::<T>(), Box::new(value));
    }

    pub fn remove<T: Any>(&mut self) -> T {
        self.map
            .remove(&TypeId::of::<T>())
            .and_then(|v| v.downcast::<T>().ok())
            .map(|v| *v)
            .unwrap()
    }

    pub fn get<F: Fetch>(&mut self) -> F::Output {
        F::get(self).unwrap()
    }
}

// Core fetch trait
pub trait Fetch {
    type Output;

    fn get(map: &mut TypeMap) -> Option<Self::Output>;
}

// Internal fetch item trait
trait FetchItem {
    type Ptr;

    fn type_id() -> TypeId;
    fn fetch_ptr(map: &mut TypeMap) -> Option<Self::Ptr>;
    unsafe fn from_ptr(ptr: Self::Ptr) -> Self;
}

// Immutable reference
impl<'a, T: 'static> FetchItem for &'a T {
    type Ptr = *const T;

    fn type_id() -> TypeId {
        TypeId::of::<T>()
    }

    fn fetch_ptr(map: &mut TypeMap) -> Option<Self::Ptr> {
        map.map
            .get(&TypeId::of::<T>())?
            .downcast_ref::<T>()
            .map(|r| r as *const T)
    }

    unsafe fn from_ptr(ptr: Self::Ptr) -> Self {
        unsafe { &*ptr }
    }
}

// Mutable reference
impl<'a, T: 'static> FetchItem for &'a mut T {
    type Ptr = *mut T;

    fn type_id() -> TypeId {
        TypeId::of::<T>()
    }

    fn fetch_ptr(map: &mut TypeMap) -> Option<Self::Ptr> {
        map.map
            .get_mut(&TypeId::of::<T>())?
            .downcast_mut::<T>()
            .map(|r| r as *mut T)
    }

    unsafe fn from_ptr(ptr: Self::Ptr) -> Self {
        unsafe { &mut *ptr }
    }
}

// Fetch a single item
impl<T> Fetch for T
where
    T: FetchItem,
{
    type Output = T;

    fn get(map: &mut TypeMap) -> Option<Self::Output> {
        T::fetch_ptr(map).map(|ptr| unsafe { T::from_ptr(ptr) })
    }
}

// Ensure that fetching multiple types does not conflict
#[cfg(debug_assertions)]
fn assert_unique(ids: &[TypeId]) {
    for i in 0..ids.len() {
        for j in i + 1..ids.len() {
            debug_assert_ne!(
                ids[i],
                ids[j],
                "duplicate type found in {}::get",
                std::any::type_name::<TypeMap>()
            );
        }
    }
}

macro_rules! impl_fetch_item {
    ($($name:ident),+) => {
        impl<$($name),+> Fetch for ($($name,)+)
        where
            $($name: FetchItem,)+
        {
            type Output = Self;

            fn get(map: &mut TypeMap) -> Option<Self> {
                #[cfg(debug_assertions)]
                assert_unique(&[ $( $name::type_id() ),+ ]);

                Some((
                    $(
                        {
                            let ptr = $name::fetch_ptr(map)?;
                            unsafe { $name::from_ptr(ptr) }
                        },
                    )+
                ))
            }
        }
    };
}

impl_fetch_item!(A);
impl_fetch_item!(A, B);
impl_fetch_item!(A, B, C);
impl_fetch_item!(A, B, C, D);
impl_fetch_item!(A, B, C, D, E);
impl_fetch_item!(A, B, C, D, E, F);
impl_fetch_item!(A, B, C, D, E, F, G);
impl_fetch_item!(A, B, C, D, E, F, G, H);

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct Foo(i32);

    #[derive(Debug)]
    struct Bar(i32);

    #[derive(Debug)]
    struct Baz(i32);

    #[test]
    fn single() {
        let mut map = TypeMap::new();
        map.insert(Foo(10));

        let foo = map.get::<&Foo>();
        assert_eq!(foo.0, 10);
    }

    #[test]
    fn multi() {
        let mut map = TypeMap::new();
        map.insert(Foo(10));
        map.insert(Bar(20));
        map.insert(Baz(30));

        let (foo, bar, baz) = map.get::<(&Foo, &mut Bar, &Baz)>();

        bar.0 = foo.0 + baz.0;

        assert_eq!(bar.0, 40);
    }

    #[test]
    #[should_panic]
    fn duplicate_types() {
        let mut map = TypeMap::new();
        map.insert(Foo(1));

        let _ = map.get::<(&Foo, &mut Foo)>();
    }
}
