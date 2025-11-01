use std::collections::HashMap;
use std::ffi::OsStr;
use std::sync::{Arc, RwLock};
use crate::target::TargetInfoParser;
use crate::tool::CompilerFamilyLookupCache;

type Env = Option<Arc<OsStr>>;

#[derive(Debug, Default)]
pub(crate) struct BuildCache {
    pub(crate) env_cache: RwLock<HashMap<Box<str>, Env>>,
    pub(crate) apple_version_cache: RwLock<HashMap<Box<str>, Arc<str>>>,
    pub(crate) cached_compiler_family: RwLock<CompilerFamilyLookupCache>,
    pub(crate) target_info_parser: TargetInfoParser,
}
