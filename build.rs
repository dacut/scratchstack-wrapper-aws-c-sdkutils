use {
    bindgen::builder,
    std::{
        env::var,
        fs::{copy, create_dir_all, read_dir},
        path::{Path, PathBuf},
    },
};

const LINK_LIBS: &str = r#"
aws-c-sdkutils
aws-c-common
"#;
const INCLUDE_PATH: &str = "aws/sdkutils";
const DEP_LIBRARIES: &str = "aws-c-common";
const FUNCTIONS: &str = r#"
aws_profile_collection_destroy
aws_profile_collection_new_from_file
aws_profile_collection_new_from_merge
aws_profile_collection_new_from_buffer
aws_profile_collection_get_profile
aws_profile_collection_get_profile_count
aws_profile_get_name
aws_profile_get_property
aws_profile_get_property_count
aws_profile_property_get_value
aws_profile_property_get_sub_property
aws_profile_property_get_sub_property_count
aws_get_credentials_file_path
aws_get_config_file_path
aws_get_profile_name
aws_endpoints_get_supported_ruleset_version
aws_endpoints_parameter_get_type
aws_endpoints_parameter_get_built_in
aws_endpoints_parameter_get_default_string
aws_endpoints_parameter_get_default_boolean
aws_endpoints_parameter_get_is_required
aws_endpoints_parameter_get_documentation
aws_endpoints_parameters_get_is_deprecated
aws_endpoints_parameter_get_deprecated_message
aws_endpoints_parameter_get_deprecated_since
aws_endpoints_ruleset_new_from_string
aws_endpoints_ruleset_acquire
aws_endpoints_ruleset_release
aws_endpoints_ruleset_get_parameters
aws_endpoints_ruleset_get_version
aws_endpoints_ruleset_get_service_id
aws_endpoints_rule_engine_new
aws_endpoints_rule_engine_acquire
aws_endpoints_rule_engine_release
aws_endpoints_request_context_new
aws_endpoints_request_context_acquire
aws_endpoints_request_context_release
aws_endpoints_request_context_add_string
aws_endpoints_request_context_add_boolean
aws_endpoints_rule_engine_resolve
aws_endpoints_resolved_endpoint_acquire
aws_endpoints_resolved_endpoint_release
aws_endpoints_resolved_endpoint_get_type","aws_endpoints_resolved_endpoint_get_url
aws_endpoints_resolved_endpoint_get_properties
aws_endpoints_resolved_endpoint_get_headers
aws_endpoints_resolved_endpoint_get_error
aws_partitions_config_new_from_string
aws_partitions_config_acquire
aws_partitions_config_release
aws_resource_name_init_from_cur
aws_resource_name_length
aws_byte_buf_append_resource_name
aws_sdkutils_library_init
aws_sdkutils_library_clean_up
"#;
const TYPES: &str = r#"
aws_profile_property
aws_profile
aws_profile_collection
aws_profile_source_type
aws_endpoints_ruleset
aws_partitions_config
aws_endpoints_parameter
aws_endpoints_rule_engine
aws_endpoints_resolved_endpoint
aws_endpoints_request_context
aws_endpoints_parameter_type
aws_endpoints_resolved_endpoint_type
aws_partitions_config
aws_resource_name
aws_sdkutils_errors
aws_sdkutils_log_subject
"#;

const VARS: &str = "";

fn get_include_dir<P: AsRef<Path>>(dir: P) -> PathBuf {
    let mut result = PathBuf::from(dir.as_ref());

    for folder in INCLUDE_PATH.split('/') {
        result.push(folder);
    }

    result
}

fn main() {
    let root = PathBuf::from(var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set"));
    let out_dir = PathBuf::from(var("OUT_DIR").expect("OUT_DIR not set"));

    let src_include_dir = root.join("include");
    let dst_include_dir = out_dir.join("include");
    let src_lib_include_dir = get_include_dir(&src_include_dir);
    let dst_lib_include_dir = get_include_dir(&dst_include_dir);
    let src_include_dir_str = src_include_dir.to_string_lossy();
    let dst_include_dir_str = dst_include_dir.to_string_lossy();
    let src_lib_include_dir_str = src_lib_include_dir.to_string_lossy();
    let dst_lib_include_dir_str = dst_lib_include_dir.to_string_lossy();

    println!("cargo:include={dst_include_dir_str}");
    println!("cargo:rerun-if-changed=include");
    println!("cargo:rerun-if-env-changed=AWS_CRT_PREFIX");

    if let Ok(aws_crt_prefix) = var("AWS_CRT_PREFIX") {
        println!("cargo:rustc-link-search={aws_crt_prefix}/lib");
    }

    for library_name in LINK_LIBS.split('\n') {
        let library_name = library_name.trim();
        if !library_name.is_empty() {
            println!("cargo:rustc-link-lib={library_name}");
        }
    }

    // Copy include files over
    create_dir_all(&dst_lib_include_dir)
        .unwrap_or_else(|e| panic!("Unable to create directory {dst_lib_include_dir_str}: {e}"));

    let mut builder = builder()
        .clang_arg(format!("-I{src_include_dir_str}"))
        .derive_debug(true)
        .derive_default(true)
        .derive_partialeq(true)
        .derive_eq(true)
        .allowlist_recursively(false); // Don't dive into dependent libraries.
    
    for dep in DEP_LIBRARIES.split('\n') {
        let dep = dep.trim();
        if dep.is_empty() {
            continue;
        }

        let dep = String::from(dep).replace('-', "_").to_uppercase();
        let dep_include_dir = PathBuf::from(var(format!("DEP_{dep}_INCLUDE")).unwrap_or_else(|_| panic!("DEP_{dep}_INCLUDE not set")));
        let dep_include_dir_str = dep_include_dir.to_string_lossy();
        builder = builder.clang_arg(format!("-I{dep_include_dir_str}"));
    }

    let mut n_includes = 0;

    for entry in read_dir(&src_lib_include_dir)
        .unwrap_or_else(|e| panic!("Unable to list header files in {src_lib_include_dir_str}: {e}"))
    {
        let entry =
            entry.unwrap_or_else(|e| panic!("Unable to read directory entry in {src_lib_include_dir_str}: {e}"));
        let file_name_string = entry.file_name();
        let src_path = src_lib_include_dir.join(&file_name_string);
        let src_path_str = src_path.to_string_lossy();
        let dst_path = dst_lib_include_dir.join(&file_name_string);

        if entry.file_type().unwrap_or_else(|e| panic!("Unable to read file type of {src_path_str}: {e}")).is_file() {
            // Only include header files ending with .h; ignore .inl.
            let file_name_utf8 = file_name_string.to_str().expect("Unable to convert file name to UTF-8");
            if file_name_utf8.ends_with(".h") {
                builder = builder.header(src_path_str.to_string());
                n_includes += 1;
            }

            // Copy all files to the output directory.
            copy(&src_path, &dst_path).unwrap_or_else(|e| {
                panic!(
                    "Failed to copy from {src_path_str} to {dst_path_str}: {e}",
                    dst_path_str = dst_path.to_string_lossy()
                )
            });
        }
    }

    if n_includes == 0 {
        panic!("No header files found in {src_lib_include_dir_str}");
    }

    for function_pattern in FUNCTIONS.split('\n') {
        let function_pattern = function_pattern.trim();
        if !function_pattern.is_empty() {
            builder = builder.allowlist_function(function_pattern);
        }
    }

    for type_pattern in TYPES.split('\n') {
        let type_pattern = type_pattern.trim();
        if !type_pattern.is_empty() {
            builder = builder.allowlist_type(type_pattern);
        }
    }

    for var_pattern in VARS.split('\n') {
        let var_pattern = var_pattern.trim();
        if !var_pattern.is_empty() {
            builder = builder.allowlist_var(var_pattern);
        }
    }

    let bindings_filename = out_dir.join("bindings.rs");
    let bindings = builder.generate().expect("Unable to generate bindings");
    bindings.write_to_file(&bindings_filename).unwrap_or_else(|e| {
        panic!(
            "Failed to write bindings to {bindings_filename_str}: {e}",
            bindings_filename_str = bindings_filename.to_string_lossy()
        )
    });

    if cfg!(any(target_os = "ios", target_os = "macos")) {
        println!("cargo:rustc-link-arg=-framework");
        println!("cargo:rustc-link-arg=CoreFoundation");
    }
}
