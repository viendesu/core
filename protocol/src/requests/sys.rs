use crate::requests::Response;
use eva::service;

use crate::types::True;

#[service(result = Response)]
pub trait Sys {
    #[args(derive(Default), non_exhaustive)]
    fn healthcheck() -> True;
}
