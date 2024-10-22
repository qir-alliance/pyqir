; ModuleID = 't_adj'
source_filename = "t_adj"

define void @main() #0 {
  call void @__quantum__qis__t__adj(ptr null)
  ret void
}

declare void @__quantum__qis__t__adj(ptr)

attributes #0 = { "entry_point" "output_labeling_schema" "qir_profiles"="custom" "required_num_qubits"="1" "required_num_results"="0" }
