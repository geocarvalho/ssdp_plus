import random
from typing import List, Set

# Assuming these are placeholders for now. We'll implement or import them later.
class Avaliador:
    @staticmethod
    def avaliarMediaDimensoes(patterns: List['Pattern'], length: int) -> float:
        return sum(len(p.items) for p in patterns) / length

class D:
    numeroItensUtilizados = 0
    itensUtilizados = []

class Const:
    random = random.Random(42)

class Pattern:
    def __init__(self, items: Set[int], tipoAvaliacao: str):
        self.items = items
        self.tipoAvaliacao = tipoAvaliacao

class INICIALIZAR:
    @staticmethod
    def D1(tipoAvaliacao: str) -> List[Pattern]:
        P0 = [Pattern(set(), tipoAvaliacao) for _ in range(D.numeroItensUtilizados)]
        for i in range(D.numeroItensUtilizados):
            itens = {D.itensUtilizados[i]}
            P0[i] = Pattern(itens, tipoAvaliacao)
        return P0

    @staticmethod
    def aleatorioD(tipoAvaliacao: str, numeroDimensoes: int, tamanhoPopulacao: int) -> List[Pattern]:
        P0 = [Pattern(set(), tipoAvaliacao) for _ in range(tamanhoPopulacao)]
        for i in range(tamanhoPopulacao):
            itens = set()
            while len(itens) < numeroDimensoes:
                itens.add(D.itensUtilizados[Const.random.randint(0, D.numeroItensUtilizados - 1)])
            P0[i] = Pattern(itens, tipoAvaliacao)
        return P0

    @staticmethod
    def aleatorio1_D_Pk(tipoAvaliacao: str, tamanhoPopulacao: int, Pk: List[Pattern]) -> List[Pattern]:
        numeroDimensoes = max(2, int(Avaliador.avaliarMediaDimensoes(Pk, len(Pk))))
        P0 = [Pattern(set(), tipoAvaliacao) for _ in range(tamanhoPopulacao)]
        i = 0
        for _ in range(9 * tamanhoPopulacao // 10):
            itens = set()
            while len(itens) < numeroDimensoes:
                itens.add(D.itensUtilizados[Const.random.randint(0, D.numeroItensUtilizados - 1)])
            P0[i] = Pattern(itens, tipoAvaliacao)
            i += 1
        itensPk = set()
        for p in Pk:
            itensPk.update(p.items)
        itensPkArray = list(itensPk)
        for j in range(i, tamanhoPopulacao):
            itens = set()
            while len(itens) < numeroDimensoes:
                if len(itensPkArray) > numeroDimensoes:
                    itens.add(itensPkArray[Const.random.randint(0, len(itensPkArray) - 1)])
                else:
                    if Const.random.random() < 0.5:
                        itens.add(itensPkArray[Const.random.randint(0, len(itensPkArray) - 1)])
                    else:
                        itens.add(D.itensUtilizados[Const.random.randint(0, D.numeroItensUtilizados - 1)])
            P0[j] = Pattern(itens, tipoAvaliacao)
        return P0

    @staticmethod
    def aleatorio1_D(tipoAvaliacao: str, limiteDimensoes: int, tamanhoPopulacao: int) -> List[Pattern]:
        P0 = [Pattern(set(), tipoAvaliacao) for _ in range(tamanhoPopulacao)]
        for i in range(tamanhoPopulacao):
            d = Const.random.randint(1, limiteDimensoes)
            itens = set()
            while len(itens) < d:
                itens.add(D.itensUtilizados[Const.random.randint(0, D.numeroItensUtilizados - 1)])
            P0[i] = Pattern(itens, tipoAvaliacao)
        return P0

    @staticmethod
    def aleatorio_1_25(tipoAvaliacao: str, tamanhoPopulacao: int) -> List[Pattern]:
        P0 = [Pattern(set(), tipoAvaliacao) for _ in range(tamanhoPopulacao)]
        dimensaoMaxima = int(D.numeroItensUtilizados * 0.25)
        for i in range(tamanhoPopulacao):
            d = Const.random.randint(0, dimensaoMaxima)
            itens = set()
            while len(itens) < d:
                itens.add(D.itensUtilizados[Const.random.randint(0, D.numeroItensUtilizados - 1)])
            P0[i] = Pattern(itens, tipoAvaliacao)
        return P0

    @staticmethod
    def aleatorioPercentualSize(tipoAvaliacao: str, tamanhoPopulacao: int, percentualGenes: float) -> List[Pattern]:
        P0 = [Pattern(set(), tipoAvaliacao) for _ in range(tamanhoPopulacao)]
        dimensao = int(percentualGenes * D.numeroItensUtilizados)
        for i in range(tamanhoPopulacao):
            itens = set()
            while len(itens) < dimensao:
                itens.add(D.itensUtilizados[Const.random.randint(0, D.numeroItensUtilizados - 1)])
            P0[i] = Pattern(itens, tipoAvaliacao)
        return P0 