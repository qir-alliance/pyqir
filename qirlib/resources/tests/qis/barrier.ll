; ModuleID = 'barrier'
source_filename = "barrier"

define void @main() #0 {
  call void @__quantum__qis__barrier__body()
  ret void
}

declare void @__quantum__qis__barrier__body()

attributes #0 = { "entry_point" "output_labeling_schema" "qir_profiles"="custom" "required_num_qubits"="0" "required_num_results"="0" }
