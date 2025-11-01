MODULE F77_FENDA

    ! Interfaces for the fenda routines

    INTERFACE FENDA

        SUBROUTINE SSKPFA(UPLO, MTHD, N, A, LDA, PFAFF, &
                IWORK, WORK, LWORK, INFO)
            USE PFAPACK_PREC, ONLY : singleprec
            CHARACTER(LEN = 1), INTENT(IN) :: UPLO, MTHD
            INTEGER, INTENT(IN) :: LDA, LWORK, N
            INTEGER, INTENT(OUT) :: INFO
            INTEGER, INTENT(OUT) :: IWORK(*)
            REAL(singleprec), INTENT(OUT) :: PFAFF
            REAL(singleprec), INTENT(INOUT) :: A(LDA, *)
            REAL(singleprec), INTENT(OUT) :: WORK(*)
        END SUBROUTINE SSKPFA

    END INTERFACE FENDA

END MODULE F77_FENDA
