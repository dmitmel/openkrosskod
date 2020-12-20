// Is this re-export necessary?
pub use std::prelude::v1::*;

pub use std::borrow::{Borrow, BorrowMut, Cow};
pub use std::cell::{Cell, Ref, RefCell, RefMut, UnsafeCell};
pub use std::cmp;
pub use std::collections::{
  BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, LinkedList, VecDeque,
};
pub use std::convert::{TryFrom, TryInto};
pub use std::env;
pub use std::ffi::{self, CStr, CString, OsStr, OsString};
pub use std::fmt;
pub use std::fs::{self, File, OpenOptions as FileOpenOptions};
pub use std::hash::{Hash, Hasher};
pub use std::io::{self, BufRead, BufReader, BufWriter, Read, Seek, Write};
pub use std::iter::{self, FromIterator};
pub use std::marker::PhantomData;
pub use std::mem;
pub use std::ops::{
  Deref, DerefMut, Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive,
};
pub use std::os::raw::*;
pub use std::path::{Path, PathBuf};
pub use std::pin::Pin;
pub use std::ptr;
pub use std::rc::{Rc, Weak as RcWeak};
pub use std::slice;
pub use std::sync::atomic::{
  AtomicBool, AtomicI16, AtomicI32, AtomicI64, AtomicI8, AtomicIsize, AtomicPtr, AtomicU16,
  AtomicU32, AtomicU64, AtomicU8, AtomicUsize, Ordering as AtomicOrdering,
};
pub use std::sync::mpsc;
pub use std::sync::{
  Arc, Barrier, BarrierWaitResult, Condvar, LockResult, Mutex, MutexGuard, Once, RwLock,
  RwLockReadGuard, RwLockWriteGuard, TryLockResult, Weak as ArcWeak,
};
pub use std::thread;
pub use std::time::{self, Duration, Instant};
pub use std::{f32, f64, str};

#[cfg(feature = "anyhow")]
pub use ::anyhow::{
  self, bail, ensure, format_err, Context as ResultContextExt, Error as AnyError,
  Result as AnyResult,
};
#[cfg(feature = "bitflags")]
pub use ::bitflags::bitflags;
#[cfg(feature = "log")]
pub use ::log::{self, debug, error, info, log, log_enabled, trace, warn, Level as LogLevel};

#[cfg_attr(not(feature = "breakpoint"), inline(always))]
pub fn breakpoint() {
  #[cfg(feature = "breakpoint")]
  {
    #[cfg(unix)]
    nix::sys::signal::raise(nix::sys::signal::SIGINT).unwrap();
  }
}
