subroutine foo() bind(C)
    print *, 'hello from fortran foo.f90'
end subroutine foo

! function bar() bind(C) result(i)
!     integer(4) :: i
!     i = 5
! end function bar
