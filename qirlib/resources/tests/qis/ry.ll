; ModuleID = 'ry'
source_filename = "ry"

%Qubit = type opaque

define void @main() #0 {
  call void @__quantum__qis__ry__body(double 0.000000e+00, %Qubit* null)
  ret void
}

declare void @__quantum__qis__ry__body(double, %Qubit*)

attributes #0 = { "entry_point" "num_required_qubits"="1" "num_required_results"="0" "output_labeling_schema" "qir_profiles"="custom" }
