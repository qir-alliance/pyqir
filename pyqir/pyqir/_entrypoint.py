# function

from typing import List, Optional
from pyqir import Module, Function, Context


def entry_point(
    module: Module,
    name: str,
    required_num_qubits: int,
    required_num_results: int,
    qir_profiles: Optional[str],
    output_labeling_schema: Optional[str],
) -> Function:
    """
    Creates an entry point.

    :param Module module: The parent module.
    :param str name: The entry point name.
    :param int required_num_qubits: The number of qubits required by the entry point.
    :param int required_num_results: The number of results required by the entry point.
    :param Optional[str] qir_profiles: Value identifying the profile the entry point has been compiled for. Use base_profile when QIR is compliant.
    :param Optional[str] output_labeling_schema: An arbitrary string value that identifies the schema used by a compiler frontend that produced the IR to label the recorded output
    :returns: An entry point.
    """
    void = pyqir.Type.void(module.context)
    function = mod.add_external_function(name, pyqir.FunctionType(void, []))


# pub unsafe fn entry_point(
#     module: LLVMModuleRef,
#     name: &CStr,
#     required_num_qubits: u64,
#     required_num_results: u64,
#     qir_profiles: &str,
#     output_labeling_schema: &str,
# ) -> LLVMValueRef {
#     let context = LLVMGetModuleContext(module);
#     let void = LLVMVoidTypeInContext(context);
#     let ty = LLVMFunctionType(void, [].as_mut_ptr(), 0, 0);
#     let function = LLVMAddFunction(module, name.as_ptr(), ty);

#     add_string_attribute(function, b"entry_point", b"");
#     add_string_attribute(
#         function,
#         b"num_required_qubits",
#         required_num_qubits.to_string().as_bytes(),
#     );
#     add_string_attribute(
#         function,
#         b"num_required_results",
#         required_num_results.to_string().as_bytes(),
#     );

#     add_string_attribute(function, b"qir_profiles", qir_profiles.as_bytes());

#     add_string_attribute(
#         function,
#         b"output_labeling_schema",
#         output_labeling_schema.as_bytes(),
#     );

#     function
# }
