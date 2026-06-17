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

    pub fn insert<T: 'static>(&mut self, value: T) {
        self.map.insert(TypeId::of::<T>(), Box::new(value));
    }

    pub fn remove<T: 'static>(&mut self) -> Option<T> {
        self.map
            .remove(&TypeId::of::<T>())
            .and_then(|v| v.downcast::<T>().ok())
            .map(|v| *v)
    }

    /// Unified multi-query API
    pub fn get<Q>(&mut self) -> Option<Q::Output>
    where
        Q: Query,
    {
        Q::get(self)
    }
}

//
// =======================================================
// Core Query trait
// =======================================================
//

pub trait Query {
    type Output;

    fn get(map: &mut TypeMap) -> Option<Self::Output>;
}

//
// =======================================================
// Internal QueryItem trait
// (turns types into raw pointers safely)
// =======================================================
//

trait QueryItem {
    type Target: 'static;
    type Ptr;

    fn type_id() -> TypeId;

    fn fetch_ptr(map: &mut TypeMap) -> Option<Self::Ptr>;

    unsafe fn from_ptr(ptr: Self::Ptr) -> Self;
}

//
// Immutable reference
//

impl<'a, T: 'static> QueryItem for &'a T {
    type Target = T;
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

//
// Mutable reference
//

impl<'a, T: 'static> QueryItem for &'a mut T {
    type Target = T;
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

//
// =======================================================
// Safety helper: prevent duplicate types in a query
// =======================================================
//

fn assert_unique(ids: &[TypeId]) {
    for i in 0..ids.len() {
        for j in i + 1..ids.len() {
            assert!(ids[i] != ids[j], "duplicate type in TypeMap::get query");
        }
    }
}

//
// =======================================================
// Macro: tuple implementations (1–8)
// =======================================================
//

impl<A> Query for A
where
    A: QueryItem,
{
    type Output = A;

    fn get(map: &mut TypeMap) -> Option<Self::Output> {
        let ids = [A::type_id()];
        assert_unique(&ids);

        unsafe {
            let ptr = A::fetch_ptr(map)?;
            Some(A::from_ptr(ptr))
        }
    }
}

macro_rules! impl_query {
    ($($name:ident),+) => {
        impl<$($name),+> Query for ($($name,)+)
        where
            $($name: QueryItem,)+
        {
            type Output = Self;

            fn get(map: &mut TypeMap) -> Option<Self> {
                let ids = [
                    $($name::type_id()),+
                ];

                assert_unique(&ids);

                unsafe {
                    Some((
                        $(
                            {
                                let ptr = $name::fetch_ptr(map)?;
                                $name::from_ptr(ptr)
                            },
                        )+
                    ))
                }
            }
        }
    };
}

impl_query!(A);
impl_query!(A, B);
impl_query!(A, B, C);
impl_query!(A, B, C, D);
impl_query!(A, B, C, D, E);
impl_query!(A, B, C, D, E, F);
impl_query!(A, B, C, D, E, F, G);
impl_query!(A, B, C, D, E, F, G, H);

//
// =======================================================
// Tests
// =======================================================
//

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

        let foo = map.get::<&Foo>().unwrap();
        assert_eq!(foo.0, 10);
    }

    #[test]
    fn multi() {
        let mut map = TypeMap::new();
        map.insert(Foo(10));
        map.insert(Bar(20));
        map.insert(Baz(30));

        let (foo, bar, baz) = map.get::<(&Foo, &mut Bar, &Baz)>().unwrap();

        bar.0 = foo.0 + baz.0;

        assert_eq!(bar.0, 40);
    }

    #[test]
    #[should_panic]
    fn duplicate_types() {
        let mut map = TypeMap::new();
        map.insert(Foo(1));

        let _ = map.get::<(&Foo, &mut Foo)>().unwrap();
    }
}
