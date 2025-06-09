import unittest
from MUTACAO import MUTACAO, Pattern, D

class TestMUTACAO(unittest.TestCase):
    def test_unGeneTrocaOuAdicionaOuExclui(self):
        # Initialize D.itensUtilizados for testing
        D.itensUtilizados = [1, 2, 3, 4, 5]
        D.numeroItensUtilizados = len(D.itensUtilizados)

        # Create a pattern with some items
        items = {1, 2, 3}
        p = Pattern(items, "test")

        # Call the method to mutate the pattern
        mutated_pattern = MUTACAO.unGeneTrocaOuAdicionaOuExclui(p, "test")

        # Check that the mutated pattern is not the same as the original
        self.assertNotEqual(mutated_pattern.getItens(), items)

if __name__ == '__main__':
    unittest.main() 