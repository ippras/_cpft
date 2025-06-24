import pandas as pd
import plotly.graph_objects as go
import plotly.express as px

# Загрузка и подготовка данных
df = pd.read_csv('_temp/source.csv')
df_cleaned = df.dropna(subset=['EquivalentChainLength', 'OnsetTemperature', 'TemperatureStep', 'FattyAcid'])
df_plot = df_cleaned.copy()

# --- Создание фигуры с помощью Graph Objects для полного контроля ---
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

    # Добавляем линию
    line_df = acid_df.sort_values(by=['TemperatureStep', 'OnsetTemperature'])
    fig.add_trace(go.Scatter3d(
        x=line_df['EquivalentChainLength'],
        y=line_df['OnsetTemperature'],
        z=line_df['TemperatureStep'],
        mode='lines',
        line=dict(width=2.5, color=acid_color),
        legendgroup=acid,
        name=acid,
        showlegend=False,
        hoverinfo='none'
    ))

    # Добавляем точки (маркеры)
    for step in acid_df['TemperatureStep'].unique():
        step_df = acid_df[acid_df['TemperatureStep'] == step]
        
        fig.add_trace(go.Scatter3d(
            x=step_df['EquivalentChainLength'],
            y=step_df['OnsetTemperature'],
            z=step_df['TemperatureStep'],
            mode='markers',
            marker=dict(
                size=5,
                color=acid_color,
                symbol=symbol_map[step],
                colorscale='Viridis',
                cmin=df_plot['OnsetTemperature'].min(),
                cmax=df_plot['OnsetTemperature'].max(),
                cauto=False,
                colorbar=dict(title="Onset Temp.")
            ),
            legendgroup=acid,
            name=acid,
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
    title='3D-график жирных кислот с возможностью множественного выбора',
    scene=dict(
        xaxis_title='Equivalent Chain Length',
        yaxis_title='Onset Temperature',
        zaxis_title='Temperature Step'
    ),
    # --- ИЗМЕНЕНИЯ ЗДЕСЬ ---
    legend=dict(
        title=dict(text='Fatty Acid (нажмите для вкл/выкл)'), # Заголовок легенды
        x=0,           # Позиция по X (0 = левый край)
        y=1,           # Позиция по Y (1 = верхний край)
        xanchor="left",  # "Привязать" левый край легенды к координате X
        yanchor="top",   # "Привязать" верхний край легенды к координате Y
        bgcolor="rgba(255, 255, 255, 0.7)", # Полупрозрачный фон для читаемости
        bordercolor="Black",
        borderwidth=1
    ),
    margin=dict(l=0, r=0, b=0, t=40)
)

# Показываем график
fig.show()