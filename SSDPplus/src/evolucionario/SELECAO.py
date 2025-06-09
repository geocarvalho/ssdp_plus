import random
from typing import List

# Assuming these are placeholders for now. We'll implement or import them later.
class Const:
    random = random.Random(42)

class Pattern:
    def __init__(self, items, tipoAvaliacao: str):
        self.items = items
        self.tipoAvaliacao = tipoAvaliacao
        self.quality = 0.0  # Placeholder for quality

    def getQualidade(self) -> float:
        return self.quality

class SELECAO:
    @staticmethod
    def proporcao25_75(tamanhoPopulacao: int) -> List[int]:
        indices = []
        i = 0
        for _ in range(int(tamanhoPopulacao * 0.75)):
            indices.append(Const.random.randint(0, tamanhoPopulacao // 4 - 1))
        for _ in range(int(tamanhoPopulacao * 0.25)):
            indices.append(Const.random.randint(0, tamanhoPopulacao - 1))
        return indices

    @staticmethod
    def torneioBinario(tamanhoPopulacao: int, P: List[Pattern]) -> List[int]:
        indices = []
        for _ in range(tamanhoPopulacao):
            indiceP1 = Const.random.randint(0, len(P) - 1)
            indiceP2 = Const.random.randint(0, len(P) - 1)
            if P[indiceP1].getQualidade() > P[indiceP2].getQualidade():
                indices.append(indiceP1)
            else:
                indices.append(indiceP2)
        return indices

    @staticmethod
    def torneioBinario(P: List[Pattern]) -> int:
        indiceP1 = Const.random.randint(0, len(P) - 1)
        indiceP2 = Const.random.randint(0, len(P) - 1)
        if P[indiceP1].getQualidade() > P[indiceP2].getQualidade():
            return indiceP1
        else:
            return indiceP2

    @staticmethod
    def selecionarMelhoresDistintos(P: List[Pattern], Pnovo: List[Pattern]) -> List[Pattern]:
        tamanhoPopulacao = len(P)
        PAsterisco = [None] * tamanhoPopulacao
        patternAux = P.copy()
        for pnovo in Pnovo:
            if SELECAO.ehInedito(pnovo, patternAux):
                patternAux.append(pnovo)
        patternAux.sort()
        for i in range(tamanhoPopulacao):
            PAsterisco[i] = patternAux[i]
        return PAsterisco

    @staticmethod
    def selecionarMelhoresDistintos(P1: List[Pattern], P2: List[Pattern], P3: List[Pattern]) -> List[Pattern]:
        tamanhoPopulacao = len(P1)
        PAsterisco = [None] * tamanhoPopulacao
        patternAux = P1.copy()
        for p2 in P2:
            if SELECAO.ehInedito(p2, patternAux):
                patternAux.append(p2)
        for p3 in P3:
            if SELECAO.ehInedito(p3, patternAux):
                patternAux.append(p3)
        patternAux.sort()
        for i in range(tamanhoPopulacao):
            PAsterisco[i] = patternAux[i]
        return PAsterisco

    @staticmethod
    def selecionarMelhores(P: List[Pattern], Pnovo: List[Pattern]) -> List[Pattern]:
        tamanhoPopulacao = len(P)
        PAsterisco = [None] * tamanhoPopulacao
        PAuxiliar = P + Pnovo
        PAuxiliar.sort()
        for i in range(tamanhoPopulacao):
            PAsterisco[i] = PAuxiliar[i]
        return PAsterisco

    @staticmethod
    def ehInedito(p: Pattern, pList: List[Pattern]) -> bool:
        for p2 in pList:
            if SELECAO.ehIgual(p, p2):
                return False
        return True

    @staticmethod
    def ehIgual(p1: Pattern, p2: Pattern) -> bool:
        return p1.items == p2.items 