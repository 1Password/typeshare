use typeshare_driver::typeshare_binary;

use typeshare_kotlin::Kotlin;
use typeshare_typescript::TypeScript;

typeshare_binary! { TypeScript, Kotlin<'config> }
