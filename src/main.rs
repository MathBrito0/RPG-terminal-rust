use std::io;
use std::time::Instant;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    execute,
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Gauge, List, ListItem, ListState, Paragraph},
    Terminal,
};

pub mod heroi;
pub mod inimigo;

// Estados da interface para sabermos o que o jogador está fazendo
#[derive(PartialEq)]
enum GameState {
    MenuAcao,
    Inventario,
    AguardandoTurnoInimigo,
    FimDeJogo,
}

// Transforma vida atual / vida máxima em um número entre 0.0 e 1.0
// (é isso que a barra Gauge precisa para saber quanto encher)
fn calcular_ratio(vida: u32, vida_max: u32) -> f64 {
    if vida_max == 0 {
        return 0.0;
    }
    // "as f64" converte para número com vírgula, senão a divisão daria 0 ou 1
    let ratio = vida as f64 / vida_max as f64;
    if ratio > 1.0 {
        return 1.0;
    }
    ratio
}

// Escolhe a cor da barra conforme a porcentagem de vida
fn cor_da_vida(ratio: f64) -> Color {
    if ratio > 0.5 {
        Color::Green
    } else if ratio > 0.25 {
        Color::Yellow
    } else {
        Color::Red
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Configura o terminal para o modo TUI (Tela alternativa e raw mode)
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 2. Inicializa o herói e o inimigo
    let mut heroi = heroi::Personagem {
        nome: String::from("Shujinko"),
        classe: String::from("Mago"),
        vida: 100,
        vida_max: 100, // NOVO
        ataque: 50,
        magia: 20,
        estamina: 40,
        inventario: vec![
            heroi::Item {
                nome: String::from("Poção"),
                descricao: String::from("Recupera sua vida."),
                quantidade: 2,
            },
            heroi::Item {
                nome: String::from("Poção de Magia"),
                descricao: String::from("Recupera sua magia."),
                quantidade: 1,
            },
            heroi::Item {
                nome: String::from("Poção de Estamina"),
                descricao: String::from("Recupera seu fôlego."),
                quantidade: 1,
            },
            heroi::Item {
                nome: String::from("Espada"),
                descricao: String::from("Sua arma favorita."),
                quantidade: 1,
            },
            heroi::Item {
                nome: String::from("Escudo"),
                descricao: String::from("Sua maior defesa."),
                quantidade: 1,
            },
        ],
        acao: true,
    };

    let mut inimigo = inimigo::Inimigo {
        nome: String::from("Chelebrimbor"),
        especie: String::from("Elfo"),
        vida: 200,
        vida_max: 200, // NOVO
        ataque: 20,
        magia: 10,
        acao: true,
    };

    let cronometro = Instant::now();

    // Variáveis de controle de jogo e combate
    let mut heroi_defesa = false;
    let mut inimigo_defesa = false;
    let mut estado_atual = GameState::MenuAcao;

    // NOVO: guarda qual item da mochila está selecionado (0 = primeiro)
    let mut item_selecionado: usize = 0;

    // Histórico de mensagens da batalha
    let mut logs: Vec<String> = vec![
        String::from("=== BEM-VINDO AO SEU RPG FAVORITO ==="),
        String::from("Combate iniciado contra Chelebrimbor! Boa sorte."),
    ];

    // 3. Loop principal da TUI
    let mut running = true;
    while running {
        // Desenha a tela
        terminal.draw(|f| {
            // Divide a tela verticalmente em 3 blocos: Status (topo), Logs (meio), Comandos (baixo)
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3), // Status (2 barras lado a lado)
                    Constraint::Min(6),    // Histórico / Logs
                    Constraint::Length(8), // Controles / Ações / Mochila
                ])
                .split(f.size());

            // --- Bloco 1: Status ---
            // Divide o bloco de cima em duas colunas: herói (esquerda) e inimigo (direita)
            let colunas = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(chunks[0]);

            // Barra do herói
            let ratio_heroi = calcular_ratio(heroi.vida, heroi.vida_max);
            let mut titulo_heroi = format!(
                " {} ({}) | Magia {} | Estamina {} ",
                heroi.nome, heroi.classe, heroi.magia, heroi.estamina
            );
            if heroi_defesa {
                titulo_heroi.push_str("[DEFENDENDO] ");
            }
            let barra_heroi = Gauge::default()
                .block(Block::default().title(titulo_heroi.as_str()).borders(Borders::ALL))
                .gauge_style(Style::default().fg(cor_da_vida(ratio_heroi)).bg(Color::Black))
                .ratio(ratio_heroi)
                .label(format!("Vida: {}/{}", heroi.vida, heroi.vida_max));
            f.render_widget(barra_heroi, colunas[0]);

            // Barra do inimigo
            let ratio_inimigo = calcular_ratio(inimigo.vida, inimigo.vida_max);
            let mut titulo_inimigo = format!(
                " {} ({}) | Magia {} ",
                inimigo.nome, inimigo.especie, inimigo.magia
            );
            if inimigo_defesa {
                titulo_inimigo.push_str("[DEFENDENDO] ");
            }
            let barra_inimigo = Gauge::default()
                .block(Block::default().title(titulo_inimigo.as_str()).borders(Borders::ALL))
                .gauge_style(Style::default().fg(cor_da_vida(ratio_inimigo)).bg(Color::Black))
                .ratio(ratio_inimigo)
                .label(format!("Vida: {}/{}", inimigo.vida, inimigo.vida_max));
            f.render_widget(barra_inimigo, colunas[1]);

            // --- Bloco 2: Histórico (Logs) ---
            // Descobre quantas linhas cabem na caixa (altura menos as 2 bordas)
            let altura_logs = chunks[1].height as usize;
            let linhas_visiveis = altura_logs.saturating_sub(2);
            // Pega só as últimas mensagens que cabem
            let inicio = if logs.len() > linhas_visiveis {
                logs.len() - linhas_visiveis
            } else {
                0
            };
            let ultimos_logs = logs[inicio..].join("\n");
            let logs_block = Paragraph::new(ultimos_logs).block(
                Block::default()
                    .title(" Acontecimentos da Batalha ")
                    .borders(Borders::ALL),
            );
            f.render_widget(logs_block, chunks[1]);

            // --- Bloco 3: Painel de baixo (muda dependendo do estado) ---
            if estado_atual == GameState::Inventario {
                // Mochila como uma lista onde dá para escolher com as setas
                let mut itens_da_lista: Vec<ListItem> = Vec::new();
                for item in heroi.inventario.iter() {
                    let linha = format!("{} (x{}) - {}", item.nome, item.quantidade, item.descricao);
                    itens_da_lista.push(ListItem::new(linha));
                }

                let lista = List::new(itens_da_lista)
                    .block(
                        Block::default()
                            .title(" Mochila (Cima/Baixo escolhe, Enter usa, Esc volta) ")
                            .borders(Borders::ALL),
                    )
                    .highlight_style(Style::default().fg(Color::Black).bg(Color::Yellow))
                    .highlight_symbol("> ");

                // O ListState guarda qual linha está marcada
                let mut estado_lista = ListState::default();
                estado_lista.select(Some(item_selecionado));
                f.render_stateful_widget(lista, chunks[2], &mut estado_lista);
            } else {
                // Nos outros estados, é só um texto
                let menu_texto = match estado_atual {
                    GameState::MenuAcao => String::from(
                        "[0] Atacar com arma    [1] Defender    [2] Usar Magia\n\
                         [3] Chamar Socorro     [4] Mochila     [5] Fugir      [6] Sair",
                    ),
                    GameState::AguardandoTurnoInimigo => String::from(
                        "Pressione [Enter] ou [Espaço] para permitir o turno do inimigo...",
                    ),
                    GameState::FimDeJogo => {
                        String::from("FIM DE JOGO! Pressione [Q] para fechar a aplicação.")
                    }
                    // Esse caso nunca chega aqui (tratado no if de cima)
                    GameState::Inventario => String::new(),
                };

                let menu_block = Paragraph::new(menu_texto).block(
                    Block::default()
                        .title(" Painel de Ações ")
                        .borders(Borders::ALL),
                );
                f.render_widget(menu_block, chunks[2]);
            }
        })?;

        // 4. Captura de Eventos do Teclado
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                // NOVO: ignora o evento de "soltar a tecla" (evita ação duplicada no Windows)
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                // Se o herói ou inimigo morreram, trava no fim de jogo
                if heroi.vida == 0 || inimigo.vida == 0 {
                    estado_atual = GameState::FimDeJogo;
                    if let KeyCode::Char('q') | KeyCode::Char('Q') = key.code {
                        running = false;
                    }
                    continue;
                }

                match estado_atual {
                    GameState::MenuAcao => {
                        match key.code {
                            KeyCode::Char('0') => {
                                // Ação: Atacar
                                logs.push(String::from("Você atacou!"));
                                if inimigo_defesa {
                                    logs.push(format!("{} estava defendendo e bloqueou parcialmente o golpe!", inimigo.nome));
                                    let dano_reduzido = heroi.ataque / 2;
                                    inimigo.vida = inimigo.vida.saturating_sub(dano_reduzido);
                                    inimigo_defesa = false;
                                } else {
                                    inimigo.vida = inimigo.vida.saturating_sub(heroi.ataque);
                                }

                                if inimigo.vida == 0 {
                                    logs.push(String::from("O inimigo foi derrotado! Você venceu!"));
                                    estado_atual = GameState::FimDeJogo;
                                } else {
                                    estado_atual = GameState::AguardandoTurnoInimigo;
                                }
                            },
                            KeyCode::Char('1') => {
                                // Ação: Defender
                                logs.push(String::from("Você se colocou em posição defensiva!"));
                                heroi_defesa = true;
                                estado_atual = GameState::AguardandoTurnoInimigo;
                            },
                            KeyCode::Char('2') => {
                                // Ação: Magia
                                logs.push(String::from("Você usou uma magia!"));
                                inimigo.vida = inimigo.vida.saturating_sub(heroi.magia);
                                if inimigo.vida == 0 {
                                    logs.push(String::from("O inimigo foi derrotado! Você venceu!"));
                                    estado_atual = GameState::FimDeJogo;
                                } else {
                                    estado_atual = GameState::AguardandoTurnoInimigo;
                                }
                            },
                            KeyCode::Char('3') => {
                                // Ação: Socorro (Mascote)
                                logs.push(String::from("Você chamou seu mascote para ajudar na batalha!"));
                                inimigo.vida = inimigo.vida.saturating_sub(10);
                                if inimigo.vida == 0 {
                                    logs.push(String::from("O inimigo foi derrotado! Você venceu!"));
                                    estado_atual = GameState::FimDeJogo;
                                } else {
                                    estado_atual = GameState::AguardandoTurnoInimigo;
                                }
                            },
                            KeyCode::Char('4') => {
                                // Ação: Abrir Mochila (sempre começa no primeiro item)
                                item_selecionado = 0;
                                estado_atual = GameState::Inventario;
                            },
                            KeyCode::Char('5') => {
                                // Ação: Fugir
                                logs.push(String::from("Você fugiu da batalha!"));
                                running = false;
                            },
                            KeyCode::Char('6') | KeyCode::Char('q') | KeyCode::Char('Q') => {
                                running = false;
                            },
                            _ => {}
                        }
                    },
                    GameState::Inventario => {
                        match key.code {
                            KeyCode::Up => {
                                // Seta para cima: sobe um item (se não estiver no primeiro)
                                if item_selecionado > 0 {
                                    item_selecionado -= 1;
                                }
                            },
                            KeyCode::Down => {
                                // Seta para baixo: desce um item (se não estiver no último)
                                if item_selecionado + 1 < heroi.inventario.len() {
                                    item_selecionado += 1;
                                }
                            },
                            KeyCode::Enter => {
                                // Só tenta usar se a mochila não estiver vazia
                                if heroi.inventario.len() > 0 {
                                    let nome_item = heroi.inventario[item_selecionado].nome.clone();
                                    match heroi::usar_pocao(&mut heroi, &nome_item) {
                                        Ok(mensagem) => {
                                            // Deu certo: mostra a mensagem e passa o turno
                                            logs.push(mensagem);
                                            estado_atual = GameState::AguardandoTurnoInimigo;
                                        },
                                        Err(mensagem) => {
                                            // Não deu certo: mostra a mensagem e continua na mochila
                                            logs.push(mensagem);
                                        },
                                    }
                                }
                            },
                            KeyCode::Esc | KeyCode::Char('b') | KeyCode::Char('B') => {
                                estado_atual = GameState::MenuAcao;
                            },
                            _ => {}
                        }
                    },
                    GameState::AguardandoTurnoInimigo => {
                        // Quando o jogador pressiona Enter ou Espaço, o turno do inimigo é executado
                        if let KeyCode::Enter | KeyCode::Char(' ') = key.code {
                            // Sorteia a ação do inimigo usando o cronômetro
                            let sorteio = cronometro.elapsed().as_millis() % 5;

                            match sorteio {
                                0 => {
                                    logs.push(format!("{} atacou você!", inimigo.nome));
                                    if heroi_defesa {
                                        logs.push(String::from("Você usou sua defesa e bloqueou parcialmente o ataque!"));
                                        let dano_reduzido = inimigo.ataque / 2;
                                        heroi.vida = heroi.vida.saturating_sub(dano_reduzido);
                                        heroi_defesa = false;
                                    } else {
                                        heroi.vida = heroi.vida.saturating_sub(inimigo.ataque);
                                    }
                                },
                                1 => {
                                    logs.push(format!("{} se defendeu!", inimigo.nome));
                                    inimigo_defesa = true;
                                },
                                2 => {
                                    logs.push(format!("{} preparou uma ação tática misteriosa!", inimigo.nome));
                                },
                                3 => {
                                    logs.push(String::from("O inimigo usou uma magia contra você!"));
                                    heroi.vida = heroi.vida.saturating_sub(inimigo.magia);
                                },
                                _ => {
                                    logs.push(String::from("O inimigo chamou reforços!"));
                                    heroi.vida = heroi.vida.saturating_sub(10);
                                },
                            }

                            if heroi.vida == 0 {
                                logs.push(String::from("Você foi derrotado... Fim de Jogo."));
                                estado_atual = GameState::FimDeJogo;
                            } else {
                                estado_atual = GameState::MenuAcao;
                            }
                        }
                    },
                    GameState::FimDeJogo => {
                        if let KeyCode::Char('q') | KeyCode::Char('Q') = key.code {
                            running = false;
                        }
                    }
                }
            }
        }
    }

    // 5. Restaura o terminal original ao fechar a aplicação
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
