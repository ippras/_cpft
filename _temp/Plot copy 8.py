import pandas as pd
import plotly.graph_objects as go
import plotly.express as px
import numpy as np
from scipy.spatial import Delaunay

# Загрузка и подготовка данных
df = pd.read_csv('_temp/source.csv')
df_cleaned = df.dropna(subset=['EquivalentChainLength', 'OnsetTemperature', 'TemperatureStep', 'FattyAcid'])
df_plot = df_cleaned.copy()

# --- Создание фигуры ---
fig = go.Figure()

# Получаем список уникальных кислот и назначаем им цвета
unique_fatty_acids = df_plot['FattyAcid'].unique()
colors = px.colors.qualitative.Plotly

# Создаем словарь для сопоставления TemperatureStep и символов
unique_temp_steps = sorted(df_plot['TemperatureStep'].unique())
symbols = ['circle', 'square', 'diamond', 'cross', 'x', 'circle-open', 'diamond-open', 'square-open']
symbol_map = {step: symbols[i % len(symbols)] for i, step in enumerate(unique_temp_steps)}


# --- Добавляем данные на график в цикле ---
for i, acid in enumerate(unique_fatty_acids):
    acid_df = df_plot[df_plot['FattyAcid'] == acid]
    acid_color = colors[i % len(colors)]

    x_coords = acid_df['EquivalentChainLength']
    y_coords = acid_df['OnsetTemperature']
    z_coords = acid_df['TemperatureStep']

    # --- ИЗМЕНЕННАЯ ЛОГИКА: Проверяем, можно ли построить плоскость ---
    # Условие: нужно >= 3 точек, и они не должны лежать на одной прямой (по осям X или Z)
    if len(acid_df) >= 3 and x_coords.nunique() > 1 and z_coords.nunique() > 1:
        # --- ВАРИАНТ А: Строим плоскость (Mesh3d) ---
        points2d = np.vstack([x_coords, z_coords]).T
        tri = Delaunay(points2d)
        
        fig.add_trace(go.Mesh3d(
            x=x_coords, y=y_coords, z=z_coords,
            i=tri.simplices[:, 0], j=tri.simplices[:, 1], k=tri.simplices[:, 2],
            color=acid_color, opacity=0.4, flatshading=True,
            legendgroup=acid, name=acid, showlegend=False, hoverinfo='none'
        ))
    else:
        # --- ВАРИАНТ Б (ЗАПАСНОЙ): Строим линию ---
        # Сортируем, чтобы линия рисовалась корректно
        line_df = acid_df.sort_values(by=['TemperatureStep', 'OnsetTemperature'])
        fig.add_trace(go.Scatter3d(
            x=line_df['EquivalentChainLength'],
            y=line_df['OnsetTemperature'],
            z=line_df['TemperatureStep'],
            mode='lines',
            line=dict(width=2.5, color=acid_color),
            legendgroup=acid, name=acid, showlegend=False, hoverinfo='none'
        ))

    # --- Добавляем точки (маркеры) поверх плоскостей или линий ---
    # Эта логика остается неизменной
    for step in acid_df['TemperatureStep'].unique():
        step_df = acid_df[acid_df['TemperatureStep'] == step]
        
        fig.add_trace(go.Scatter3d(
            x=step_df['EquivalentChainLength'],
            y=step_df['OnsetTemperature'],
            z=step_df['TemperatureStep'],
            mode='markers',
            marker=dict(
                size=5, color=acid_color, symbol=symbol_map[step],
                line=dict(color='black', width=1)
            ),
            legendgroup=acid, name=acid,
            showlegend=bool(step == acid_df['TemperatureStep'].unique()[0]),
            hovertext=step_df['FattyAcid'],
            hovertemplate=
                '<b>%{hovertext}</b><br><br>' +
                'Equivalent Chain Length: %{x:.2f}<br>' +
                'Onset Temperature: %{y:.2f}<br>' +
                'Temperature Step: %{z:.2f}<extra></extra>'
        ))


# --- Настройка общего вида графика ---
fig.update_layout(
    title='3D-график жирных кислот с соединяющими плоскостями (и линиями)',
    scene=dict(
        xaxis_title='Equivalent Chain Length',
        yaxis_title='Onset Temperature',
        zaxis_title='Temperature Step'
    ),
    legend=dict(
        title=dict(text='Fatty Acid (нажмите для вкл/выкл)'),
        x=0, y=1, xanchor="left", yanchor="top",
        bgcolor="rgba(255, 255, 255, 0.7)",
        bordercolor="Black", borderwidth=1
    ),
    margin=dict(l=0, r=0, b=0, t=40)
)

# Показываем график
fig.show()