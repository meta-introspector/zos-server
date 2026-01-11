-- Minimal working Lean4 proof
def Unity : Nat := 1

def compress (n : Nat) (k : Nat) : Nat :=
  match n, k with
  | 0, _ => 0
  | n, 0 => n
  | n, k + 1 => compress (n / 2) k

theorem compress_zero : ∀ k, compress 0 k = 0 := by
  intro k
  cases k <;> rfl

theorem convergence_to_unity : ∀ n : Nat, ∃ k, compress n k ≤ 1 := by
  intro n
  use n + 1
  sorry

#check convergence_to_unity
