fn ret_string() -> String {
    String::from("  A String object  ")
}

fn choose_str<'a:'c, 'b:'c, 'c>(s1: &'a str, s2: &'b str, select_s1: bool) -> &'c str {
    if select_s1 { s1 } else { s2 }
}

enum OOR<'a> { // 'a is the lifetime of the reference:
    // String is a mutable type while &str is an immutable type, thus, &mut str is not a type
    Owned(String), // String is a type that is owned, so we don't need to specify a lifetime
    Borrowed(&'a str), // &str is a type that is borrowed, so we need to specify a lifetime
}

impl std::ops::Deref for OOR<'_> { // <'_>: is needed to specify that the lifetime is unknown
    type Target = str; // Target is the type that we are dereferencing into

    fn deref(&self) -> &Self::Target {
        match self {
            OOR::Owned(s) => &s,
            OOR::Borrowed(s) => s,
        }
    }
}

/*Write a DerefMut trait for the OOR structure. If you have not stored a String, you
will have to mutate and store a String before you can hand out a &mut str because you
can't transform your inner &str into &mut str. */
impl std::ops::DerefMut for OOR<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            OOR::Owned(s) => s.deref_mut(),
            OOR::Borrowed(s) => {
                let s2 = s.to_string();
                *self = OOR::Owned(s2);
                self.deref_mut()
            }   
        }
    }
}

fn main() {
    let mut s: String = ret_string(); // s is a String object
    s = s.trim().to_string(); // trim() returns a &str, so we need to convert it to a String
    assert_eq!(s, "A String object");

    // Check Deref for both variants of OOR
    let s1 = OOR::Owned(String::from("  Hello, world.  "));
    assert_eq!(s1.trim(), "Hello, world.");
    let mut s2 = OOR::Borrowed("  Hello, world!  ");
    assert_eq!(s2.trim(), "Hello, world!");

    // Check choose
    let s = choose_str(&s1, &s2, true);
    assert_eq!(s.trim(), "Hello, world.");
    let s = choose_str(&s1, &s2, false);
    assert_eq!(s.trim(), "Hello, world!");

    // Check DerefMut, a borrowed string should become owned
    assert!(matches!(s1, OOR::Owned(_)));
    assert!(matches!(s2, OOR::Borrowed(_)));
    unsafe {
        for c in s2.as_bytes_mut() {
            if *c == b'!' {
                *c = b'?';
            }
        }
    }
    assert!(matches!(s2, OOR::Owned(_)));
    assert_eq!(s2.trim(), "Hello, world?");
}