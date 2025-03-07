use once_cell::sync::Lazy;
use std::collections::HashSet;
use std::sync::Mutex;

static NODE_IDS_IN_USE: Lazy<Mutex<HashSet<u32>>> = Lazy::new(|| {
    Mutex::new(HashSet::new())
});

pub fn register_id(id: u32) -> Result<(), String> {
    let mut set = NODE_IDS_IN_USE.lock().unwrap();
    if set.contains(&id) {
        Err(format!("ID {} já está em uso", id))
    } else {
        set.insert(id);
        Ok(())
    }
}

pub fn unregister_id(id: u32) {
    let mut set = NODE_IDS_IN_USE.lock().unwrap();
    set.remove(&id);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_unique_id() {
        let id = 1234;
        
        // Cleanup prévio: se tiver ficado no set de outro teste
        unregister_id(id);

        // 1) Registra pela primeira vez: deve ser Ok
        let res = register_id(id);
        assert!(res.is_ok(), "Registrar ID {} pela primeira vez deve ser Ok", id);

        // 2) Registrar novamente o mesmo ID => Err
        let res2 = register_id(id);
        assert!(res2.is_err(), "Registrar ID {} duplicado deve gerar Err", id);

        // Cleanup
        unregister_id(id);
    }

    #[test]
    fn test_unregister_then_register() {
        let id = 5678;
        // Limpamos qualquer registro anterior
        unregister_id(id);

        // 1) Registra
        assert!(register_id(id).is_ok());
        // 2) Desregistra
        unregister_id(id);

        // 3) Registra de novo, deve ser Ok
        let res2 = register_id(id);
        assert!(res2.is_ok(), "Depois de unregister, deve poder registrar novamente");
        
        // Cleanup
        unregister_id(id);
    }

    #[test]
    fn test_unregister_non_existent() {
        let id = 8888;
        // Se não está registrado, unregister_id(id) deve simplesmente não fazer nada
        // Verificamos que não dá panic
        unregister_id(id);
    }

    // (Opcional) Teste de concorrência
    // Este teste cria threads que tentam registrar o mesmo ID simultaneamente,
    // e confirma que só um deles consegue, enquanto os outros falham.
    #[test]
    fn test_concurrent_register() {
        use std::thread;

        let id = 9999;
        // Limpamos antes
        unregister_id(id);

        // Cria um vetor de threads que tentam registrar
        let mut handles = vec![];
        for _ in 0..5 {
            let handle = thread::spawn(move || {
                register_id(id)
            });
            handles.push(handle);
        }

        let mut success_count = 0;
        let mut error_count = 0;

        for h in handles {
            let result = h.join().unwrap(); // unwrap => resultado do register_id
            if result.is_ok() {
                success_count += 1;
            } else {
                error_count += 1;
            }
        }

        // Esperado: somente 1 thread consegue registrar, as demais falham
        assert_eq!(success_count, 1, "Apenas uma thread deve conseguir registrar o ID");
        assert_eq!(error_count, 4, "As outras devem falhar");

        // Cleanup
        unregister_id(id);
    }
}
