; ModuleID = 'mz'
source_filename = "mz"

define void @main() #0 {
  call void @__quantum__qis__mz__body(ptr null, ptr null)
  ret void
}

declare void @__quantum__qis__mz__body(ptr, ptr writeonly) #1

attributes #0 = { "entry_point" "output_labeling_schema" "qir_profiles"="custom" "required_num_qubits"="1" "required_num_results"="1" }
attributes #1 = { "irreversible" }
