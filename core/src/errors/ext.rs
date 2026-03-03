use crate::{
    errors::{Aux, Generic, auth::InvalidRole},
    types::user::Role,
};

pub trait AuthzRoleExt {
    fn require_at_least(self, role: Role) -> Result<(), InvalidRole>;
}

impl AuthzRoleExt for Role {
    fn require_at_least(self, role: Role) -> Result<(), InvalidRole> {
        if self >= role {
            Ok(())
        } else {
            Err(InvalidRole {
                required_at_least: role,
            })
        }
    }
}

pub trait ResultExt: Sized {
    type Ok;
    type Err;

    fn aux_err(self) -> Result<Self::Ok, Aux>
    where
        Self::Err: Into<Aux>;
}

impl<O, E> ResultExt for Result<O, E> {
    type Ok = O;
    type Err = E;

    #[track_caller]
    fn aux_err(self) -> Result<O, Aux>
    where
        E: Into<Aux>,
    {
        match self {
            Self::Ok(o) => Ok(o),
            Self::Err(e) => Err(e.into()),
        }
    }
}

pub trait BoolExt: Sized {
    fn true_or<E>(self, err: E) -> Result<(), E>;
    fn false_or<E>(self, err: E) -> Result<(), E>;
}

impl BoolExt for bool {
    fn true_or<E>(self, err: E) -> Result<(), E> {
        if self { Ok(()) } else { Err(err) }
    }

    fn false_or<E>(self, err: E) -> Result<(), E> {
        if self { Err(err) } else { Ok(()) }
    }
}

pub trait OptExt: Sized {
    type Inside;

    fn none_or<E>(self, f: impl FnOnce(Self::Inside) -> E) -> Result<(), E>;
}

impl<T> OptExt for Option<T> {
    type Inside = T;

    fn none_or<E>(self, f: impl FnOnce(Self::Inside) -> E) -> Result<(), E> {
        match self {
            Some(v) => Err(f(v)),
            None => Ok(()),
        }
    }
}

pub trait ErrExt: Sized {
    #[track_caller]
    fn aux(self) -> Aux
    where
        Self: Into<Aux>,
    {
        self.into()
    }

    #[track_caller]
    fn spec<S>(self) -> Generic<S>
    where
        S: From<Self>,
    {
        Generic::Spec(self.into())
    }
}

impl<T> ErrExt for T {}
