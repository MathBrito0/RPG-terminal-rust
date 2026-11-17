pub struct Personagem {
    pub nome: String,
    pub classe: String,
    pub vida: u32,
    pub vida_max: u32, // NOVO: guarda a vida máxima (usada na barra e para limitar a cura)
    pub ataque: u32,
    pub magia: u32,
    pub estamina: u32,
    pub inventario: Vec<Item>,
    pub acao: bool,
}

pub struct Item {
    pub nome: String,
    pub descricao: String,
    pub quantidade: u32,
}

// Agora a função NÃO usa println! (isso bagunça a tela da TUI).
// Em vez disso, ela devolve um Result<String, String>:
//   Ok(mensagem)  -> o item foi usado com sucesso (o turno do jogador gasta)
//   Err(mensagem) -> não deu para usar o item (o jogador continua na mochila)
pub fn usar_pocao(heroi: &mut Personagem, nome_pocao: &str) -> Result<String, String> {
    // Percorre TODOS os itens da mochila procurando pelo nome
    for item in &mut heroi.inventario {
        if item.nome == nome_pocao {
            // Achou o item. Primeiro vê se ainda tem unidades
            if item.quantidade == 0 {
                return Err(format!("Você não tem mais {}!", item.nome));
            }

            if item.nome == "Poção" {
                item.quantidade -= 1;
                heroi.vida += 5;
                // Não deixa a vida passar do máximo
                if heroi.vida > heroi.vida_max {
                    heroi.vida = heroi.vida_max;
                }
                return Ok(String::from("Você usou Poção! Vida recuperada."));
            } else if item.nome == "Poção de Magia" {
                item.quantidade -= 1;
                heroi.magia += 10;
                return Ok(String::from("Você usou Poção de Magia! Magia recuperada."));
            } else if item.nome == "Poção de Estamina" {
                item.quantidade -= 1;
                heroi.estamina += 4;
                return Ok(String::from("Você usou Poção de Estamina! Fôlego recuperado."));
            } else {
                // Espada, Escudo etc. não são consumíveis
                return Err(format!("{} não pode ser usado assim.", item.nome));
            }
        }
    }

    // Se o loop terminou sem achar nada
    Err(String::from("Item não encontrado!"))
}
