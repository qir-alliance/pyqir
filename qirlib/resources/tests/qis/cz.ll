; ModuleID = 'cz'
source_filename = "cz"

define void @main() #0 {
  call void @__quantum__qis__cz__body(ptr null, ptr inttoptr (i64 1 to ptr))
  ret void
}

declare void @__quantum__qis__cz__body(ptr, ptr)

attributes #0 = { "entry_point" "output_labeling_schema" "qir_profiles"="custom" "required_num_qubits"="2" "required_num_results"="0" }
