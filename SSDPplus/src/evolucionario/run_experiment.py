import pandas as pd
from MUTACAO import MUTACAO, Pattern, D

def load_csv_data(file_path):
    # Load the CSV file into a pandas DataFrame
    data = pd.read_csv(file_path)
    # Assuming the first column is the target value, adjust as necessary
    D.valorAlvo = data.iloc[:, -1].unique()[0]  # Example: using the last column as the target
    D.dadosStr = data.values.tolist()
    D.numeroExemplos = len(data)
    D.numeroAtributos = len(data.columns) - 1  # Assuming the last column is the target
    D.itensUtilizados = list(range(D.numeroAtributos))  # Example: using all attributes
    D.numeroItensUtilizados = len(D.itensUtilizados)

def main():
    # Specify the path to the CSV file
    csv_file_path = 'experiments/BIO-19-rep10-SSDPplusxSSDPxSDxSDrssxDSSD_TabelaoSemKzero.csv'
    # Load the CSV data
    load_csv_data(csv_file_path)
    # Example: Create a pattern and mutate it
    items = {1, 2, 3}  # Example items
    p = Pattern(items, "test")
    mutated_pattern = MUTACAO.unGeneTrocaOuAdicionaOuExclui(p, "test")
    print("Mutated Pattern Items:", mutated_pattern.getItens())

if __name__ == '__main__':
    main() 