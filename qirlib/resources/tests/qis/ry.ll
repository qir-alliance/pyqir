; ModuleID = 'ry'
source_filename = "ry"

%Qubit = type opaque

define void @main() #0 {
entry:
  call void @__quantum__qis__ry__body(double 0.000000e+00, %Qubit* null)
  ret void
}

declare void @__quantum__qis__ry__body(double, %Qubit*)

attributes #0 = { "EntryPoint" "requiredQubits"="1" "requiredResults"="0" }
