; ModuleID = 'rx'
source_filename = "rx"

%Qubit = type opaque

define void @main() #0 {
  call void @__quantum__qis__rx__body(double 0.000000e+00, %Qubit* null)
  ret void
}

declare void @__quantum__qis__rx__body(double, %Qubit*)

attributes #0 = { "EntryPoint" "requiredQubits"="1" "requiredResults"="0" }
