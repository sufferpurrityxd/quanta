#![allow(dead_code)]
mod id;
#[cfg(test)]
mod test;

/// All files that have been added to access the
/// network are called artifacts.
///
/// An artifact is a file that is
/// distributed directly from the user's computer. This means that we do not save
/// the file in any way and do not convert it locally. All hash calculations, magnet
/// links, etc. happen at runtime. This approach has pros and cons.
///
/// Cons - the file may disappear due to the user's inattention
/// Pros - we do not create a new instance and do not clog memory once again
type _TODO = usize;
