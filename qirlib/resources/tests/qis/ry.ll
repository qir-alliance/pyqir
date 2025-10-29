; ModuleID = 'ry'
source_filename = "ry"

define void @main() #0 {
  call void @__quantum__qis__ry__body(double 0.000000e+00, ptr null)
  ret void
}

declare void @__quantum__qis__ry__body(double, ptr)

attributes #0 = { "entry_point" "output_labeling_schema" "qir_profiles"="custom" "required_num_qubits"="1" "required_num_results"="0" }
