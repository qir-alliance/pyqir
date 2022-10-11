; ModuleID = 't_adj'
source_filename = "t_adj"

%Qubit = type opaque

define void @main() #0 {
entry:
  call void @__quantum__qis__t__adj(%Qubit* null)
  ret void
}

declare void @__quantum__qis__t__adj(%Qubit*)

attributes #0 = { "EntryPoint" "requiredQubits"="1" "requiredResults"="0" }
