; ModuleID = 'result_record_output'
source_filename = "result_record_output"

define void @main() #0 {
  call void @__quantum__rt__result_record_output(ptr null, ptr null)
  ret void
}

declare void @__quantum__rt__result_record_output(ptr, ptr)

attributes #0 = { "entry_point" "output_labeling_schema" "qir_profiles"="custom" "required_num_qubits"="0" "required_num_results"="1" }
