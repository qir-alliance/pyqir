; ModuleID = 'rz'
source_filename = "rz"

%Qubit = type opaque

define void @main() #0 {
  call void @__quantum__qis__rz__body(double 0.000000e+00, %Qubit* null)
  ret void
}

declare void @__quantum__qis__rz__body(double, %Qubit*)

attributes #0 = { "entry_point" "output_labeling_schema" "qir_profiles"="custom" "required_num_qubits"="1" "required_num_results"="0" }
