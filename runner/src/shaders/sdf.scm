(display "Hello Scheme\n")

(define (disk r)
  (lambda (p)
    (~> (length p)
        (- r))))

(define (torus r)
  (lambda (p)
    (~> (length p)
        (- r)
        (abs)
        (- (* r 0.3)))))

(define sdf (torus 0.3))

(define (run-sdf)
  (let lp ([i 0] [j 0])
    (cond
     ((= j NUM_Y) grid)
     ((= i NUM_X) (lp 0 (+ j 1)))
     (else
      (let ([p (grid->point i j)])
        (grid-set grid i j (sdf p))
        (lp (+ 1 i) j))))))
