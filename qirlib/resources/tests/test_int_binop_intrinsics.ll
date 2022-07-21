; ModuleID = 'test_int_binop_intrinsics'
source_filename = "test_int_binop_intrinsics"

declare i32 @source()

declare void @sink_i1(i1)

declare void @sink_i32(i32)

define void @main() #0 {
entry:
  %0 = call i32 @source()
  %1 = call i32 @source()
  %2 = and i32 %0, %1
  call void @sink_i32(i32 %2)
  %3 = or i32 %0, %1
  call void @sink_i32(i32 %3)
  %4 = xor i32 %0, %1
  call void @sink_i32(i32 %4)
  %5 = add i32 %0, %1
  call void @sink_i32(i32 %5)
  %6 = sub i32 %0, %1
  call void @sink_i32(i32 %6)
  %7 = mul i32 %0, %1
  call void @sink_i32(i32 %7)
  %8 = shl i32 %0, %1
  call void @sink_i32(i32 %8)
  %9 = lshr i32 %0, %1
  call void @sink_i32(i32 %9)
  %10 = icmp eq i32 %0, %1
  call void @sink_i1(i1 %10)
  %11 = icmp ne i32 %0, %1
  call void @sink_i1(i1 %11)
  %12 = icmp ugt i32 %0, %1
  call void @sink_i1(i1 %12)
  %13 = icmp uge i32 %0, %1
  call void @sink_i1(i1 %13)
  %14 = icmp ult i32 %0, %1
  call void @sink_i1(i1 %14)
  %15 = icmp ule i32 %0, %1
  call void @sink_i1(i1 %15)
  %16 = icmp sgt i32 %0, %1
  call void @sink_i1(i1 %16)
  %17 = icmp sge i32 %0, %1
  call void @sink_i1(i1 %17)
  %18 = icmp slt i32 %0, %1
  call void @sink_i1(i1 %18)
  %19 = icmp sle i32 %0, %1
  call void @sink_i1(i1 %19)
  ret void
}

attributes #0 = { "EntryPoint" }
