#!/bin/bash

# Define an associative array of test names and their corresponding prefixes
declare -A tests_with_prefixes


tests_with_prefixes["can_generate_algebraic_enum"]="OP"
tests_with_prefixes["can_generate_generic_enum"]="Core"
tests_with_prefixes["can_generate_generic_struct"]="Core"
tests_with_prefixes["can_generate_generic_type_alias"]="Core"
tests_with_prefixes["can_generate_simple_enum"]="TypeShare"
tests_with_prefixes["test_algebraic_enum_case_name_support"]="OP"
tests_with_prefixes["can_apply_prefix_correctly"]="OP"
tests_with_prefixes["can_generate_empty_algebraic_enum"]="OP"
tests_with_prefixes["anonymous_struct_with_rename"]="Core"
tests_with_prefixes["can_handle_serde_rename"]="TypeShareX_"
tests_with_prefixes["test_type_alias"]="OP"
tests_with_prefixes["test_serialized_as"]="OP"
tests_with_prefixes["test_serialized_as_tuple"]="OP"
tests_with_prefixes["can_handle_serde_rename_on_top_level"]="OP"
tests_with_prefixes["can_handle_unit_type"]="Equatable"
tests_with_prefixes["const_enum_decorator"]="OP"
tests_with_prefixes["algebraic_enum_decorator"]="OP"
tests_with_prefixes["struct_decorator"]="OP"
tests_with_prefixes["resolves_qualified_type"]="Core"

# Iterate over the tests and create the corresponding typeshare.toml files
for test_name in "${!tests_with_prefixes[@]}"; do
    prefix="${tests_with_prefixes[$test_name]}"
    dir="$test_name"
    file="$dir/typeshare.toml"

    # Create the directory if it doesn't exist
    mkdir -p "$dir"

    # Write the typeshare.toml file
    cat > "$file" <<EOF
[swift]
prefix = "$prefix"
EOF

    echo "Created $file with prefix '$prefix'"
done
