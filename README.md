# blair_mountain

A crate that exports one macro (`union`) to create types that are enums with checks in debug mode, but unions in release mode. In debug mode, invalid access will panic, while in release
they will not.  It is therefore unsafe. Fields should be treated as fields of unions (i.e
no non-`Copy` types allowed, etc). **Please test all code generated with `blair_mountain` in
both release and debug modes.**
