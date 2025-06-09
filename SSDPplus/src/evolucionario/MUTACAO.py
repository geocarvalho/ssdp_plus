import random
from typing import List, Set

# Assuming these are placeholders for now. We'll implement or import them later.
class Const:
    random = random.Random(42)

class D:
    nomeBase = ""
    caminho = ""
    numeroExemplos = 0
    numeroExemplosPositivo = 0
    numeroExemplosNegativo = 0
    numeroAtributos = 0
    numeroItens = 0
    SEPARADOR = ","
    nomeVariaveis = []
    itemAtributo = []
    itemValor = []
    itemAtributoStr = []
    itemValorStr = []
    Dp = []
    Dn = []
    itensUtilizados = []
    numeroItensUtilizados = 0
    dadosStr = []
    tipoDiscretizacao = ""
    valorAlvo = ""
    valoresAlvo = []

class Pattern:
    def __init__(self, items: Set[int], tipoAvaliacao: str):
        self.items = items
        self.tipoAvaliacao = tipoAvaliacao

    def getItens(self) -> Set[int]:
        return self.items

class MUTACAO:
    @staticmethod
    def unGeneTrocaOuAdicionaOuExclui(p: Pattern, tipoAvaliacao: str) -> Pattern:
        itens = p.getItens()
        if not itens:
            itens.add(D.itensUtilizados[random.randint(0, D.numeroItensUtilizados - 1)])
            return Pattern(itens, tipoAvaliacao)
        novoItens = set()
        r = random.random()
        if r < 0.33:
            indiceExcluir = random.randint(0, len(itens) - 1)
            for i, item in enumerate(itens):
                if i != indiceExcluir:
                    novoItens.add(item)
        elif r > 0.66:
            indiceExcluir = random.randint(0, len(itens) - 1)
            for i, item in enumerate(itens):
                if i != indiceExcluir:
                    novoItens.add(item)
            while len(novoItens) < len(itens):
                novoItens.add(D.itensUtilizados[random.randint(0, D.numeroItensUtilizados - 1)])
        else:
            novoItens.update(itens)
            while len(novoItens) < len(itens) + 1:
                novoItens.add(D.itensUtilizados[random.randint(0, D.numeroItensUtilizados - 1)])
        return Pattern(novoItens, tipoAvaliacao)

    @staticmethod
    def unGeneTrocaOuAdicionaOuExcluiPop(P: List[Pattern], tamanhoPopulacao: int, tipoAvaliacao: str) -> List[Pattern]:
        Pm = [None] * tamanhoPopulacao
        for i in range(tamanhoPopulacao):
            Pm[i] = MUTACAO.unGeneTrocaOuAdicionaOuExclui(P[i], tipoAvaliacao)
        return Pm

    @staticmethod
    def unGeneD(p: Pattern, tipoAvaliacao: str) -> Pattern:
        itens = p.getItens().copy()
        novoItens = set()
        indiceExcluir = random.randint(0, len(itens) - 1)
        for i, item in enumerate(itens):
            if i != indiceExcluir:
                novoItens.add(item)
        while len(novoItens) < len(itens):
            novoItens.add(D.itensUtilizados[random.randint(0, D.numeroItensUtilizados - 1)])
        return Pattern(novoItens, tipoAvaliacao) 