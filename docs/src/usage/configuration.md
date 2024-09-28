# Configuration

The behaviour of Typeshare can be customized by either passing options on the command line or in a configuration file. For any command line option that corresponds to a value in the configuration file, specifying the option on the command line will override the value in the configuration file.

## Command Line Options

- `-l`, `--lang`
    (Required) The language you want your definitions to be generated in. Currently, this option can be set to either `kotlin`, `swift`, `go`, or `typescript`.
- `-o`, `--output-file`
    (Required or -d) The file path to which the generated definitions will be written.
- `-d`, `--directory`
    (Required or -o) The folder path to write the multiple module files to.

- `-s`, `--swift-prefix`
    Specify a prefix that will be prepended to type names when generating types in Swift.

- `-M`, `--module-name`
    Specify the name of the Kotlin module for generated Kotlin source code.

- `-t`, `--target-os`
    Optional comma separated list of target os targets. Types that are restricted via `#[cfg(target_os = <target>]`
    that do not match the argument list will be filtered out.

- `-j`, `--java-package`
    Specify the name of the Java package for generated Kotlin types.

- `-c`, `--config-file`
    Instead of searching for a `typeshare.toml` file, this option can be set to specify the path to the configuration file that Typeshare will use.

- `-g`, `--generate-config-file`
    Instead of running Typeshare with the provided options, generate a configuration file called `typeshare.toml` containing the options currently specified as well as default configuration parameters.
- `--directories`
    A list argument that you can pass any number of glob patterns to. All folders and files given will be searched recursively, and all Rust sources found will be used to create a singular language source file.
- `--go-package`
    The name of the Go package for use with building for Go. This will be included in the header of the output file. This option will only be available if `typeshare-cli` was built with the `go` feature.

## Configuration File

 By default, Typeshare will look for a file called `typeshare.toml` in your current directory or any of its parent directories. Typeshare configuration files will look like this:
 ```toml
[swift]
prefix = 'MyPrefix'

[kotlin]
module_name = 'myModule'
package = 'com.example.package'

[swift.type_mappings]
"DateTime" = "Date"

[typescript.type_mappings]
"DateTime" = "string"

[kotlin.type_mappings]
"DateTime" = "String"
 ```

In the configuration file, you can specify the options you want to set so that they do not need to be specified when running Typeshare from the command line. You can also define custom type mappings to specify the foreign type that a given Rust type will correspond to.

In order to create a config file you can run the following command to generate one in your current directory.
```
typeshare -g
```
