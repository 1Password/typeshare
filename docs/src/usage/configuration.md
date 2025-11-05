# Configuration

Your Typeshare can be customized by either passing options on the command line or in a configuration file. The `typeshare-driver` will automatically create command line flags from the Config struct in your language. For any command line option that corresponds to a value in the configuration file, specifying the option on the command line will override the value in the configuration file.

Some base commands are available in any Typeshare implementation's CLI

## Base Command Line Options

- `-l`, `--lang`
    (Required) The language you want your definitions to be generated in. This will be one of the languages supported by the Typeshare you are using.
- `-o`, `--output-file`
    (Required or -d) The file path to which the generated definitions will be written.
- `-d`, `--directory`
    (Required or -o) The folder path to write the multiple module files to.

- `-t`, `--target-os`
    Optional comma separated list of target os targets. Types that are restricted via `#[cfg(target_os = <target>]`
    that do not match the argument list will be filtered out.

- `-c`, `--config`, `--config-file`
    Instead of searching for a `typeshare.toml` file, this option can be set to specify the path to the configuration file that Typeshare will use.

## Custom Config Command Line Options

To see what specific flags you can use from your implementation's config implementation, run:
```bash
your-typeshare-binary --help
```
for a full list of possible flags.

## Configuration File

 By default, Typeshare will look for a file called `typeshare.toml` in your current directory or any of its parent directories. Typeshare configuration files will look like something this:
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

In the configuration file, you can specify the options you want to set so that they do not need to be specified when running Typeshare from the command line. These options will correspond to the Config struct for the Typeshare language implementation.
