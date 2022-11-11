use crate::{aws_sdkutils_library_clean_up, aws_sdkutils_library_init};
use scratchstack_wrapper_aws_c_common::aws_default_allocator;

#[test]
fn test_init_uninit() {
    unsafe {
        aws_sdkutils_library_init(aws_default_allocator());
        aws_sdkutils_library_clean_up();
    }
}
