; ModuleID = 'ccx'
source_filename = "ccx"

define void @main() #0 {
  call void @__quantum__qis__ccx__body(ptr null, ptr inttoptr (i64 1 to ptr), ptr inttoptr (i64 2 to ptr))
  ret void
}

declare void @__quantum__qis__ccx__body(ptr, ptr, ptr)

attributes #0 = { "entry_point" "output_labeling_schema" "qir_profiles"="custom" "required_num_qubits"="3" "required_num_results"="0" }
