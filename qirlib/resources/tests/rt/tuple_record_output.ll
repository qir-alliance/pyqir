; ModuleID = 'tuple_record_output'
source_filename = "tuple_record_output"

define void @main() #0 {
  call void @__quantum__rt__tuple_record_output(i64 0, i8* null)
  ret void
}

declare void @__quantum__rt__tuple_record_output(i64, i8*)

attributes #0 = { "entry_point" "num_required_qubits"="0" "num_required_results"="0" "output_labeling_schema" "qir_profiles"="custom" }
