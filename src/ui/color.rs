macro_rules! color_fn {
    ($name:ident, $code:literal) => {
        pub fn $name(s: &str) -> String {
            format!("\x1b[{}m{s}\x1b[0m", $code)
        }
    };
}

color_fn!(red, 31);
color_fn!(green, 32);
color_fn!(yellow, 33);
color_fn!(cyan, 36);
color_fn!(gray, 90);
color_fn!(bold, 1);
color_fn!(bold_cyan, "1;36");
