//! # OsStrTools
//!
//! `OsStrTools` adds some useful methods to `OsStr` and `OsString`
//! that are missing in the standard library, like
//! [`split()`](trait.OsStrTools.html#tymethod.split),
//! [`replace()`](trait.OsStrTools.html#tymethod.replace), or
//! [`splice()`](trait.OsStrTools.html#tymethod.splice). It is mostly
//! useful for dealing dealing with things like file names, command
//! line arguments, `PathBuf`, and the like.
//!
//! Windows support is experimental, but shoud hopefully mostly work,
//! although it is not well tested and likely somewhat slower due to
//! some overhead since it requires checking the strings for
//! correctness. The checking is done by
//! [`os_str_bytes`](https://crates.io/crates/os_str_bytes).
//!
//! This crate is still maintained, but mostly has been deprecated in
//! favor of [`bstr`](https://crates.io/crates/bstr). If any bugs
//! remain it's likely they will be fixed as they come up, though.

use itertools::{Itertools, Position};
use std::ffi::{OsStr, OsString};
#[cfg(not(target_os = "windows"))]
use std::os::unix::ffi::{OsStrExt, OsStringExt};


/// Temporary storage for the string to be spliced. Can assemble the
/// final [`OsString`] in different ways, depending on the use-case.
/// Created by [`splice()`](trait.OsStrTools.html#tymethod.splice).
pub struct StringSplicer<'a, T: Bytes> {
    string: Vec<&'a OsStr>,
    to: &'a [T],
}

impl<'a, T: Bytes> StringSplicer<'a, T> {
    fn new(string: Vec<&'a OsStr>, to: &'a [T]) -> Self {
        Self {
            string,
            to
        }
    }

    /// Assembles the final string by appending individual items
    /// without any other processing.
    pub fn assemble(self) -> OsString
    {
        self.assemble_with_sep_and_wrap("", "")
    }

    /// Places a separator between all the elements that are spliced
    /// into the string.
    pub fn assemble_with_sep(self, sep: impl Bytes) -> OsString
    {
        self.assemble_with_sep_and_wrap(sep, "")
    }

    /// Wraps all items with the provided value before inserting them
    /// into the string.
    pub fn assemble_with_wrap(self, wrap: impl Bytes) -> OsString
    {
        self.assemble_with_sep_and_wrap("", wrap)
    }

    /// Wraps all items with the provided value and places the
    /// separator item between the individual elements.
    /// # Examples
    ///
    /// ```
    /// use std::ffi::OsStr;
    /// use osstrtools::{OsStrTools, StringSplicer};
    ///
    /// let strings: &[&OsStr] = &[OsStr::new("/foo/bar"),
    ///                            OsStr::new("/baz/")];
    /// let s = OsStr::new("rm #");
    ///
    /// let mut splicer = s.splice("#", strings);
    /// let result = splicer.assemble_with_sep_and_wrap(" ", "'");
    ///
    /// assert_eq!(result, "rm '/foo/bar' '/baz/'");
    /// ```
    pub fn assemble_with_sep_and_wrap(self,
                                      sep: impl Bytes,
                                      wrap: impl Bytes) -> OsString
    {
        // If len is 0, string was empty
        if self.string.len() == 0 {
            return OsString::new();
        }

        // No splits means no pattern was found
        if self.string.len() == 1 {
            return self.string[0].to_os_string();
        }

        let sep = sep.as_byte_slice();
        let wrap = wrap.as_byte_slice();
        let part_count = self.string.len();
        let to_count = self.to.len();

        let splice_items = self.to.into_iter()
            .enumerate()
            .fold(OsString::new(), |mut splice_items, (i, item)| {
                splice_items.push(wrap.bytes_as_os_str());
                splice_items.push(item.bytes_as_os_str());
                splice_items.push(wrap.bytes_as_os_str());

                // Don't add separator for last item
                if i+1 < to_count {
                    splice_items.push(sep.bytes_as_os_str());
                }
                splice_items
            });


        self.string
            .into_iter()
            .take(part_count-1)
            .fold(OsString::new(), |mut parts, part| {
                parts.push(part);
                parts.push(&splice_items);

                parts
            })
    }
}



/// Conversion Trait implemented for many byte sources that can be used as
/// arguments for search patterns and replacement strings. The methods
/// probably shouldn't be called anywhere outside this crate.
pub trait Bytes: std::fmt::Debug
{
    /// Returns the underlying buffer as a `&[u8]` slice.
    ///
    /// # WARNING
    ///
    /// Windows support requires transmuting the [`OsStr`] into a
    /// slice and back, since there is no other way to directly access
    /// the underlying bytes. This should be fine for what this
    /// library does, since all inputs are valid UTF8/WTF8 anyway and
    /// all this library does is combine or split bytes at valid
    /// edges. One exception is the possibility to supply arbitrarily
    /// prepared slices as inputs, but even then, all strings are
    /// checked for validity before being transmuted back into `OsStr`
    /// so there is no chance to end up with a broken string.
    fn as_byte_slice(&self) -> &[u8];

    /// Don't use this outside of this crate.
    ///
    /// # WARNING
    ///
    /// Transmutes arbitrary byte slices into `OsStr` on Windows which
    /// could cause bad things to happen. To prevent this, the strings
    /// are checked for validity before the conversion and if they
    /// contain invalid bytes a panic will ensure.
    ///
    /// # Panics
    ///
    /// On Windows this can panic when called on a malformed
    /// collection of bytes.
    fn bytes_as_os_str(&self) -> &OsStr {
        let bytes = self.as_byte_slice();
        OsStr::from_bytes(bytes)
    }
}


impl Bytes for &[u8] {
    fn as_byte_slice(&self) -> &[u8]  {
        self
    }
}

impl Bytes for Vec<u8> {
    fn as_byte_slice(&self) -> &[u8]  {
        self.as_slice()
    }
}

impl Bytes for &Vec<u8> {
    fn as_byte_slice(&self) -> &[u8]  {
        self.as_slice()
    }
}

impl Bytes for OsStr {
    fn as_byte_slice(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl Bytes for &OsStr {
    fn as_byte_slice(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl Bytes for &&OsStr {
    fn as_byte_slice(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl Bytes for OsString {
    fn as_byte_slice(&self) -> &[u8] {
        self.as_os_str().as_byte_slice()
    }
}

impl Bytes for &OsString {
    fn as_byte_slice(&self) -> &[u8] {
        self.as_os_str().as_byte_slice()
    }
}

impl Bytes for str {
    fn as_byte_slice(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl Bytes for &str {
    fn as_byte_slice(&self) -> &[u8] {
        self.as_bytes()
    }
}



/// Concatenation of things that can be turned into an [`Iterator`] of
/// elements that implement [`Bytes`].
pub trait OsStrConcat {
    /// Concatenates all the strings and places the separator between
    /// all elements. The separator can be an empty string.
    /// # Examples
    ///
    /// ```
    /// use std::ffi::OsStr;
    /// use osstrtools::OsStrConcat;
    ///
    /// let strings: &[&OsStr] = &[OsStr::new("a"),
    ///                            OsStr::new("b"),
    ///                            OsStr::new("c")];
    /// let result = strings.concat(" ");
    ///
    /// assert_eq!(result, "a b c");
    /// ```
    ///
    fn concat(self, sep: impl Bytes) -> OsString;
}

impl<I> OsStrConcat for I
where
    I: IntoIterator,
    I::Item: Bytes + Clone,
{
    fn concat(self, sep: impl Bytes) -> OsString {
        let sep = sep.as_byte_slice();

        let mut string = self.into_iter()
            .fold(Vec::new(), |mut string,  part| {
                string.extend_from_slice(part.as_byte_slice());
                string.extend_from_slice(sep);

                string
            });

        // Remove last separator
        string.truncate(string.len() - sep.len());

        OsString::from_vec(string.to_vec())
    }
}



/// Includes some of the methods of [`OsStrTools`] but they take the
/// [`OsString`] by value and return it without modification if the
/// specific search pattern isn't found.
///
/// Allocation is also avoided by reusing parts of the old string when
/// possible. As such, the methods here can be slightly faster and
/// might be preferable in some cases.
pub trait OsStringTools {
    /// Replace all matches with something else.
    ///
    ///If no matches are found the returned string contains the whole
    /// original string.
    fn replace(self, pat: impl Bytes, to: impl Bytes) -> OsString;

    /// Replace double quote characters in the string with `"\\\""` for
    /// safe handling over to a shell.
    fn escape_double_quote(self) -> OsString;

    /// Replace single quote characters in the string with `"\\''"` for
    /// safe handling over to a shell.
    fn escape_single_quote(self) -> OsString;

    /// Remove all matches of the pattern until a different character
    /// is found.
    ///
    /// Returns the rest.
    fn trim_start(self, pat: impl Bytes) -> OsString;

    /// Remove all matches of the pattern from the end until a
    /// different character is found.
    ///
    /// Returns the rest.
    fn trim_end(self, pat: impl Bytes) -> OsString;
}

impl OsStringTools for OsString {
    #[inline]
    fn replace(self, pat: impl Bytes, to: impl Bytes) -> OsString {
        let pat = pat.as_byte_slice();
        if self.contains(pat) {
            self.as_os_str().replace(pat, to)
        } else {
            self
        }
    }

    #[inline]
    fn escape_double_quote(self) -> OsString {
        if self.contains("\"") {
            self.as_os_str().escape_double_quote()
        } else {
            self
        }
    }

    #[inline]
    fn escape_single_quote(self) -> OsString {
        if self.contains("'") {
            self.as_os_str().escape_single_quote()
        } else {
            self
        }
    }

    #[inline]
    fn trim_start(self, pat: impl Bytes) -> OsString {
        let pat = pat.as_byte_slice();

        let skip_n = *self.as_bytes()
            .chunks(pat.len())
            .enumerate()
            .take_while(|(_, part)| part == &pat)
            .map(|(i,_)| i)
            .collect_vec()
            .last()
            .unwrap_or(&0);

        // First byte is something else already
        if skip_n == 0 {
            return self;
        } else {
            let mut string = self.into_vec();
            // Move bytes around and remove the matched bytes
            string.rotate_left(skip_n+1);
            string.truncate((string.len() - skip_n) - 1);

            return OsString::from_vec(string);
        }
    }

    #[inline]
    fn trim_end(self, pat: impl Bytes) -> OsString {
        let pat = pat.as_byte_slice();
        let mut string = self.into_vec();

        while string.ends_with(pat) {
            string.truncate(string.len() - pat.len())
        }

        OsString::from_vec(string)
    }
}

/// Extension Trait for OsStr to make working OsStr them more
/// ergonomic. It contains many methods that work similarly to those
/// of [`String`].
pub trait OsStrTools
{
    /// Split the string by looking for pat.
    ///
    /// Splits don't include the pattern itself. Like the stdlib
    /// multiple consecutive matches result in empty strings placed in
    /// between.
    ///
    /// If the string starts with the pattern the first element will
    /// be an empty string. If the string ends with the pattern, there
    /// will be an empty string at the end.
    fn split(&self, pat: impl Bytes) -> Vec<&OsStr>;

    /// Splits lines by looking for `"\n"`.
    ///
    /// Same as `.split("\n")`.
    fn split_lines(&self) -> Vec<&OsStr>;

    /// Replace all matches with something else.
    ///
    ///If no matches are found the returned string contains the whole
    /// original string.
    fn replace(&self, pat: impl Bytes, to: impl Bytes) -> OsString;

    /// Remove all matches of the pattern until a different character
    /// is found.
    ///
    /// Returns the rest.
    fn trim_start(&self, pat: impl Bytes) -> &OsStr;

    /// Remove all matches of the pattern from the end until a
    /// different character is found.
    ///
    /// Returns the rest.
    fn trim_end(&self, pat: impl Bytes) -> &OsStr;

    /// Check if the string contains the pattern.
    fn contains(&self, pat: impl Bytes) -> bool;

    /// Looks through the string for the pattern.
    ///
    /// The position of the first match is returned. If no matches are
    /// found it returns `None`.
    fn position(&self, pat: impl Bytes) -> Option<usize>;

    /// Similar to replace, but instead of inserting a single
    /// replacement string this allows inserting multiple strings at
    /// the match position.
    ///
    /// Returns a [`StringSplicer`] which allows to specify how the
    /// strings should be inserted.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::ffi::OsStr;
    /// use osstrtools::{OsStrTools, StringSplicer};
    ///
    /// let strings: &[&OsStr] = &[OsStr::new("is"),
    ///                            OsStr::new("an"),
    ///                            OsStr::new("example")];
    /// let s = OsStr::new("this #");
    ///
    /// let mut splicer: StringSplicer<'_, _> = s.splice("#", strings);
    /// let result = splicer.assemble_with_sep(" ");
    ///
    /// assert_eq!(result, "this is an example");
    /// ```
    fn splice<'a, B: Bytes>(&'a self,
                            from: impl Bytes,
                            to: &'a [B]) -> StringSplicer<'a, B>;

    /// Wraps the string with double quote characters.
    fn quote_double(&self) -> OsString;

    /// Wraps the string with single quote characters.
    fn quote_single(&self) -> OsString;

    /// Replace double quote characters in the string with `"\\\""` for
    /// safe handling over to a shell.
    fn escape_double_quote(&self) -> OsString;

    /// Replace single quote characters in the string with `"\\''"` for
    /// safe handling over to a shell.
    fn escape_single_quote(&self) -> OsString;
}



impl OsStrTools for OsStr
{
    fn split(&self, pat: impl Bytes) -> Vec<&OsStr>
    {
        let orig_string = self.as_byte_slice();
        let pat = pat.as_byte_slice();
        let pat_len = pat.len();
        let mut last_pos = 0;

        let split_string = orig_string
            .windows(pat_len)
            .enumerate()
            .scan(0, |prev_split_end, (i, chars)| {
                if chars == pat {
                    let split_at = *prev_split_end;
                    *prev_split_end = i+pat_len;
                    Some(Some((split_at, i)))
                } else {
                    Some(None)
                }
            })
            .filter_map(|s| s)
            .map(|(start, end)| {
                let split = &orig_string[start..end];
                (OsStr::from_bytes(split), end)
            })
            .with_position()
            .batching(move |it| {
                match it.next() {
                    Some(item) => {
                        let string = match item {
                            Position::First((string, _))  |
                            Position::Middle((string, _)) => string,
                            Position::Last((string, pos)) |
                            Position::Only((string, pos)) => {
                                last_pos = pos ;
                                string
                            }
                        };

                        return Some(string)
                    }
                    None => {
                        // No position was updated. Either NO or ONLY pattern in string.
                        if last_pos == 0 {
                            last_pos = self.len();
                            // If they're the same it means there was only a pattern
                            if self.as_byte_slice() == pat.as_byte_slice() {
                                return Some(OsStr::new(""));
                            } else if self.len() == 0 {
                                return None;
                            } else {
                                return Some(self);
                            }
                        } else if last_pos+pat_len <= self.len() {
                            // There is still some of the string left
                            let last = last_pos;
                            let remaining = self.len() - last_pos;
                            last_pos = self.len();
                            let split = &orig_string[last+pat_len..last+remaining];
                            return Some(OsStr::from_bytes(split));
                        } else { return None; }
                    }
                }
            })
            .collect();

        split_string
    }

    fn replace(&self, pat: impl Bytes, with: impl Bytes) -> OsString
    {
        if self.len() == 0 {
            return OsString::new()
        }

        let string_len = self.len();
        let replace_len = with.as_byte_slice().len();
        let replace_iter = std::iter::repeat(OsStr::from_bytes(with.as_byte_slice()));

        let splits = self.split(pat);
        let split_len = splits.len();

        let splits = splits
            .into_iter()
            .interleave_shortest(replace_iter);

        let mut new_string = OsString::with_capacity(string_len + replace_len);

        for split in splits.take(split_len*2-1) {
            new_string.push(split);
        }

        new_string
    }

    fn split_lines(&self) -> Vec<&OsStr> {
        let newline = "\n";
        self.split(newline)
    }

    fn quote_double(&self) -> OsString {
        let string_len = self.len();
        let quote = "\"";

        let mut new_string = OsString::with_capacity(string_len + 2);
        new_string.push(quote);
        new_string.push(self);
        new_string.push(quote);

        new_string
    }

    fn quote_single(&self) -> OsString
    {
        let string = self;
        let string_len = string.len();
        let quote = OsStr::new("\'");

        let mut new_string = OsString::with_capacity(string_len + 2);
        new_string.push(quote);
        new_string.push(string);
        new_string.push(quote);

        new_string
    }

    fn splice<'a, B: Bytes>(&'a self,
                            pat: impl Bytes,
                            to: &'a [B]) -> StringSplicer<'a, B> {
        let string = self.split(pat);
        StringSplicer::new(string, to)
    }


    fn trim_start(&self, pat: impl Bytes) -> &OsStr {
        let mut string = self.as_bytes();
        let pat = pat.as_byte_slice();
        let pat_len = pat.len();

        while string.starts_with(pat) {
            string = &string[pat_len..];
        }

        OsStr::from_bytes(string)
    }

    fn trim_end(&self, pat: impl Bytes) -> &OsStr {
        let mut string = self.as_bytes();
        let pat = pat.as_byte_slice();
        let pat_len = pat.len();

        while string.ends_with(pat) {
            string = &string[..string.len()-pat_len];
        }

        OsStr::from_bytes(string)
    }

    fn contains(&self, pat: impl Bytes) -> bool {
        self.position(pat).is_some()
    }

    fn position(&self, pat: impl Bytes) -> Option<usize> {
        let string = self.as_bytes();
        let pat = pat.as_byte_slice();
        let pat_len = pat.len();

        string.windows(pat_len)
            .position(|chars| chars == pat)
    }

    fn escape_double_quote(&self) -> OsString {
        let quote = "\"";
        let quote_escaped = "\\\"";

        self.replace(quote, quote_escaped)
    }

    fn escape_single_quote(&self) -> OsString {
        let single_quote = "'";
        let single_quote_escaped = "'\\''";

        self.replace(single_quote, single_quote_escaped)
    }
}



// These traits are used on Windows instead of OsStrExt/OsStringExt
// from the stdlib. They provide the same functionality as on *nix
// systems by using a combination of os_str_bytes' functionality and
// some use of transmutation to borrow the byte slice of OsStr and to
// turn a byte slice into an &OsStr. Strings are always checked before
// transmutation and invalid strings will cause a panic.


#[cfg(target_os = "windows")]
trait WinOsString {
    fn from_vec(s: Vec<u8>) -> OsString;
    fn into_vec(self) -> Vec<u8>;
}

#[cfg(target_os = "windows")]
impl WinOsString for OsString {
    fn from_vec(s: Vec<u8>) -> OsString {
        use os_str_bytes::OsStringBytes;
        // Verify the string. Panics if it's not valid WTF8.
        OsStringBytes::from_vec(s.clone())
            .expect("INVALID STRING: Tried to convert invalid WTF8 to OsString!")
    }

    fn into_vec(self) -> Vec<u8> {
        use os_str_bytes::OsStringBytes;
        OsStringBytes::into_vec(self)
    }
}

#[cfg(target_os = "windows")]
#[doc(cfg(windows))]
#[doc(cfg(features = "windows"))]
#[cfg(target_os = "windows")]
#[cfg(features = "windows")]
/// This uses unsafe transmutation. However, the lifetimes of passed
/// in values are respected and all byte slices are checked before
/// being turned into an `OsStr`. For example, this fails to compile:
///
/// ```compile_fail
/// use std::ffi::{OsString, OsStr};
/// use crate::osstrtools::{Bytes, WinOsStringExt, WinOsStrExt};
/// fn test_lifetime() {
///     let s: &[u8] = {
///         let string = OsString::new();
///         test_lifetime1(string)
///     };
///
///     s.len();
/// }
///
/// fn test_lifetime1(s: OsString) -> &'static [u8] {
///     let byte_vec = s.into_vec();
///     let byte_slice = byte_vec.as_byte_slice();
///     let osstr = OsStr::from_bytes(byte_slice);
///
///     let bytes = osstr.as_bytes();
///     bytes
/// }
/// ```
///
/// Similarly, this will panic (on Windows):
///
/// ```should_panic
/// #[cfg(target_os = "windows")]
/// #[cfg(features = "windows")]
/// fn will_panic() {
///     use std::ffi::{OsString, OsStr};
///     use crate::osstrtools::{Bytes, WinOsStringExt, WinOsStrExt};
///     let bytes: &[u8] = b"\xF1foo\xF1\x80bar\xF1\x80\x80baz";
///     OsStr::from_bytes(bytes);
/// }
/// ```
trait WinOsStr {
    fn from_bytes(bytes: &[u8]) -> &OsStr;
    fn as_bytes(&self) -> &[u8];
}

#[cfg(target_os = "windows")]
impl WinOsStr for OsStr {
    fn from_bytes(bytes: &[u8]) -> &OsStr {
        use os_str_bytes::OsStrBytes;

        // Verify the string. Panics if it's not valid WTF8.
        let _str: std::borrow::Cow<'_, OsStr> = OsStrBytes::from_bytes(bytes)
            .expect("INVALID STRING: Tried to convert invalid WTF8 to OsStr!");

        // Since it's a valid OsStr this should be fine...
        unsafe { (bytes as *const _).cast() }
    }

    fn as_bytes<'s>(&'s self) -> &'s [u8] {
        // This should be fine in any case, as OsStr is just a &[u8]
        unsafe { (bytes as *const _).cast::<()>() }
    }
}



#[test]
fn testsplit() {
    let s = OsStr::new("a,d,g");
    assert_eq!(s.split(","), vec!["a", "d", "g"]);
    let s = OsStr::new("abcd defe geh");
    assert_eq!(s.split(" "), vec!["abcd", "defe", "geh"]);
    let s = OsStr::new("abcd $s");
    assert_eq!(s.split("$s"), vec!["abcd ", ""]);
    let s = OsStr::new("abcd $s$s");
    assert_eq!(s.split("$s"), vec!["abcd ", "", ""]);
    let s = OsStr::new("");
    assert_eq!(s.split("$s"), Vec::<&OsStr>::new());
    let s = OsStr::new("$s");
    assert_eq!(s.split(",,"), &["$s"]);

}

#[test]
fn testreplace() {
    let s = OsStr::new("a,d,g");
    assert_eq!(s.replace(",", "WUT"), "aWUTdWUTg");
    let s = OsStr::new("a,d,g");
    assert_eq!(s.replace(".", "WUT"), "a,d,g");
    let s = OsStr::new("a,,d,,g");
    assert_eq!(s.replace(",,", "WUT"), "aWUTdWUTg");
    let s = OsStr::new("");
    assert_eq!(s.replace(",,", "WUT"), "");
    let s = OsStr::new("$s");
    assert_eq!(s.replace(",,", "WUT"), "$s");
    let s = OsString::from("a,d,g");
    assert_eq!(s.replace(",", "WUT"), "aWUTdWUTg");
    let s = OsString::from("a,d,g");
    assert_eq!(s.replace(".", "WUT"), "a,d,g");
    let s = OsString::from("a,,d,,g");
    assert_eq!(s.replace(",,", "WUT"), "aWUTdWUTg");
    let s = OsString::from("");
    assert_eq!(s.replace(",,", "WUT"), "");
    let s = OsString::from("$s");
    assert_eq!(s.replace(",,", "WUT"), "$s");
}

#[test]
fn testtrimstart() {
    let s = OsStr::new(",,,abcd,defe,geh,,,,");
    assert_eq!(s.trim_start(","), "abcd,defe,geh,,,,");
    let s = OsString::from(",,,abcd,defe,geh");
    assert_eq!(s.trim_start(","), "abcd,defe,geh");
}

#[test]
fn testtrimend() {
    let s = OsStr::new(",,,abcd,defe,geh,,,,");
    assert_eq!(s.trim_end(","), ",,,abcd,defe,geh");
    let s = OsStr::new(",,,abcd,defe,geh,,,,");
    assert_eq!(s.trim_end(","), ",,,abcd,defe,geh");
    let s = OsStr::new(",,,abcd");
    assert_eq!(s.trim_end(","), ",,,abcd");
    let s = OsStr::new(",,,abcd,,");
    assert_eq!(s.trim_end(","), ",,,abcd");
    let s = OsStr::new(",,,");
    assert_eq!(s.trim_end(","), "");
    let s = OsStr::new(",,,");
    assert_eq!(s.trim_end(",,"), ",");
    let s = OsStr::new(",,,");
    assert_eq!(s.trim_end(",,,"), "");
    let s = OsStr::new("");
    assert_eq!(s.trim_end(",,,"), "");

    let s = OsString::from(",,,abcd,defe,geh,,,,");
    assert_eq!(s.trim_end(","), ",,,abcd,defe,geh");
    let s = OsString::from(",,,abcd,defe,geh,,,,");
    assert_eq!(s.trim_end(","), ",,,abcd,defe,geh");
    let s = OsString::from(",,,abcd");
    assert_eq!(s.trim_end(","), ",,,abcd");
    let s = OsString::from(",,,abcd,,");
    assert_eq!(s.trim_end(","), ",,,abcd");
    let s = OsString::from(",,,");
    assert_eq!(s.trim_end(","), "");
    let s = OsString::from(",,,");
    assert_eq!(s.trim_end(",,"), ",");
    let s = OsString::from(",,,");
    assert_eq!(s.trim_end(",,,"), "");
    let s = OsString::from("");
    assert_eq!(s.trim_end(",,,"), "");
}


#[test]
fn testescape() {
    let s = OsStr::new("te'st");
    assert_eq!(s.escape_single_quote(), "te'\\''st");
    let s = OsStr::new("'te'st'");
    assert_eq!(s.escape_single_quote(), "'\\''te'\\''st'\\''");
    let s = OsString::from("te'st");
    assert_eq!(s.escape_single_quote(), "te'\\''st");
    let s = OsString::from("'te'st'");
    assert_eq!(s.escape_single_quote(), "'\\''te'\\''st'\\''");
}

#[test]
fn test_escape_double() {
    let s = OsStr::new("te\"st");
    assert_eq!(s.escape_double_quote(), "te\\\"st");
    let s = OsStr::new("\"te\"st\"");
    assert_eq!(s.escape_double_quote(), "\\\"te\\\"st\\\"");
    let s = OsString::from("te\"st");
    assert_eq!(s.escape_double_quote(), "te\\\"st");
    let s = OsString::from("\"te\"st\"");
    assert_eq!(s.escape_double_quote(), "\\\"te\\\"st\\\"");
}

#[test]
fn testsplice() {
    let s = OsStr::new("ls $s");
    assert_eq!(s.splice("$s", &["TEST"]).assemble_with_sep(" "),
               "ls TEST");
    let s = OsStr::new("ls $s");
    assert_eq!(s.splice("$s", &["TEST", "TEST"]).assemble_with_sep(" "),
               "ls TEST TEST");
    let s = OsStr::new("ls $s$s");
    assert_eq!(s.splice("$s", &["TEST", "TEST"]).assemble_with_sep(" "),
               "ls TEST TESTTEST TEST");
    let s = OsStr::new("");
    assert_eq!(s.splice("$s", &["TEST", "TEST"]).assemble_with_sep(" "), "");
    let s = OsStr::new("ls $s");
    assert_eq!(s.splice("$s", &[""]).assemble_with_wrap("'"), "ls ''");
    let s = OsStr::new("ls $s");
    assert_eq!(s.splice("$s", &["TEST"]).assemble_with_wrap("'"),
               "ls 'TEST'");
    let s = OsStr::new("ls $s");
    assert_eq!(s.splice("$s", &["TEST", "TEST"]).assemble_with_sep_and_wrap(" ", "'"),
               "ls 'TEST' 'TEST'");
    let s = OsStr::new("ls $s$s");
    assert_eq!(s.splice("$s", &["TEST", "TEST"]).assemble_with_sep_and_wrap(" ", "'"),
               "ls 'TEST' 'TEST''TEST' 'TEST'");
}

#[test]
fn testconcat() {
    let s = &[OsStr::new("1"), OsStr::new("2"), OsStr::new("3")];
    assert_eq!(s.concat(" "), "1 2 3");
    assert_eq!(s.concat(""), "123");
    assert_eq!(s.concat("' '"), "1' '2' '3");
    let s = &[OsStr::new("")];
    assert_eq!(s.concat(""), "");
    assert_eq!(s.concat("' '"), "");
    let s: &[&OsStr] = &[];
    assert_eq!(s.concat(""), "");
}
