; ModuleID = 'array_record_output'
source_filename = "array_record_output"

define void @main() #0 {
  call void @__quantum__rt__array_record_output(i64 0, ptr null)
  ret void
}

declare void @__quantum__rt__array_record_output(i64, ptr)

attributes #0 = { "entry_point" "output_labeling_schema" "qir_profiles"="custom" "required_num_qubits"="0" "required_num_results"="0" }
