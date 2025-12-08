; ModuleID = 'read_result'
source_filename = "read_result"

define void @main() #0 {
  %1 = call i1 @__quantum__qis__read_result__body(ptr null)
  ret void
}

declare i1 @__quantum__qis__read_result__body(ptr)

attributes #0 = { "entry_point" "output_labeling_schema" "qir_profiles"="custom" "required_num_qubits"="1" "required_num_results"="1" }
