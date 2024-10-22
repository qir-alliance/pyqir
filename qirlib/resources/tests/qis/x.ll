; ModuleID = 'x'
source_filename = "x"

define void @main() #0 {
  call void @__quantum__qis__x__body(ptr null)
  ret void
}

declare void @__quantum__qis__x__body(ptr)

attributes #0 = { "entry_point" "output_labeling_schema" "qir_profiles"="custom" "required_num_qubits"="1" "required_num_results"="0" }
