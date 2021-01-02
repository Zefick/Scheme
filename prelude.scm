
(define (foldr func end lst)
  (if (null? lst)
      end
      (func (car lst) (foldr func end (cdr lst)))))

(define (foldl func accum lst)
  (if (null? lst)
      accum
      (foldl func (func accum (car lst)) (cdr lst))))

(define (append list1 list2)
   (foldr cons list2 list1))

(define (reverse items)
  (foldr (lambda (x r) (append r (list x))) '() items))
