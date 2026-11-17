pub struct Inimigo {
    pub nome: String,
    pub especie: String,
    pub vida: u32,
    pub vida_max: u32, // NOVO: usada para calcular a barra de vida
    pub ataque: u32,
    pub magia: u32,
    pub acao: bool,
}
