; ModuleID = 'z'
source_filename = "z"

%Qubit = type opaque

define void @main() #0 {
  call void @__quantum__qis__z__body(%Qubit* null)
  ret void
}

declare void @__quantum__qis__z__body(%Qubit*)

attributes #0 = { "EntryPoint" "requiredQubits"="1" "requiredResults"="0" }
