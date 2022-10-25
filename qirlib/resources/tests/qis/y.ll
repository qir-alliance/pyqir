; ModuleID = 'y'
source_filename = "y"

%Qubit = type opaque

define void @main() #0 {
entry:
  call void @__quantum__qis__y__body(%Qubit* null)
  ret void
}

declare void @__quantum__qis__y__body(%Qubit*)

attributes #0 = { "EntryPoint" "requiredQubits"="1" "requiredResults"="0" }
