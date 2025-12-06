; ModuleID = 'wheel_module'
source_filename = "wheel_module"

@_fmt_ld = constant [5 x i8] c"%ld\0A\00"
@_fmt_s = constant [4 x i8] c"%s\0A\00"
@_fmt_scan = constant [7 x i8] c"%255s\00\00"
@_input_buf = global [256 x i8] zeroinitializer
@str_For_in_range_test_ = constant [20 x i8] c"For in range test:\00\00"
@str_Loop_from_0_to_5_ = constant [19 x i8] c"Loop from 0 to 5:\00\00"
@str_Loop_from_2_to_7_ = constant [19 x i8] c"Loop from 2 to 7:\00\00"
@str_Done = constant [6 x i8] c"Done\00\00"

declare i32 @printf(ptr, ...)

declare i32 @scanf(ptr, ...)

declare ptr @malloc(i64)

declare i32 @atoi(ptr)

declare i32 @strcmp(ptr, ptr)

define i32 @main(i32 %0, ptr %1) {
entry:
  %call_user_main = call i64 @user_main()
  ret i64 0
}

define i64 @user_main() {
entry:
  %call_printf = call i32 (ptr, ...) @printf(ptr @_fmt_s, ptr @str_For_in_range_test_)
  %call_printf1 = call i32 (ptr, ...) @printf(ptr @_fmt_s, ptr @str_Loop_from_0_to_5_)
  %for_i = alloca i64, align 8
  store i64 0, ptr %for_i, align 4
  br label %for_check_i

for_check_i:                                      ; preds = %for_body_i, %entry
  %load_i = load i64, ptr %for_i, align 4
  %for_cond = icmp slt i64 %load_i, 5
  br i1 %for_cond, label %for_body_i, label %for_after_i

for_body_i:                                       ; preds = %for_check_i
  %load_i2 = load i64, ptr %for_i, align 4
  %call_printf3 = call i32 (ptr, ...) @printf(ptr @_fmt_ld, i64 %load_i2)
  %loadinc_i = load i64, ptr %for_i, align 4
  %inc_i = add i64 %loadinc_i, 1
  store i64 %inc_i, ptr %for_i, align 4
  br label %for_check_i

for_after_i:                                      ; preds = %for_check_i
  %call_printf4 = call i32 (ptr, ...) @printf(ptr @_fmt_s, ptr @str_Loop_from_2_to_7_)
  %for_j = alloca i64, align 8
  store i64 2, ptr %for_j, align 4
  br label %for_check_j

for_check_j:                                      ; preds = %for_body_j, %for_after_i
  %load_j = load i64, ptr %for_j, align 4
  %for_cond5 = icmp slt i64 %load_j, 7
  br i1 %for_cond5, label %for_body_j, label %for_after_j

for_body_j:                                       ; preds = %for_check_j
  %load_j6 = load i64, ptr %for_j, align 4
  %call_printf7 = call i32 (ptr, ...) @printf(ptr @_fmt_ld, i64 %load_j6)
  %loadinc_j = load i64, ptr %for_j, align 4
  %inc_j = add i64 %loadinc_j, 1
  store i64 %inc_j, ptr %for_j, align 4
  br label %for_check_j

for_after_j:                                      ; preds = %for_check_j
  %call_printf8 = call i32 (ptr, ...) @printf(ptr @_fmt_s, ptr @str_Done)
  ret i64 0
}
