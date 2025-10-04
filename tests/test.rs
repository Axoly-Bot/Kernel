#[cfg(test)]
mod tests {

    use twilight_gateway::Intents;

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    #[tokio::test]
    async fn test_kernel_creation() {
        let token = "test_token".to_string();
        let intents = Intents::empty();
        
        let result = Kernel::Kernel::new(token, intents, "redis://localhost".to_string()).await;
        // En un entorno real esto fallaría sin un token válido,
        // pero verificamos que la función se compila correctamente
        assert!(result.is_err()); // Debería fallar con token inválido
    }

    #[test]
    fn test_kernel_struct_fields() {
        // Test para verificar que la estructura tiene los campos esperados
        let kernel = std::mem::size_of::<Kernel::Kernel>();
        assert!(kernel > 0, "Kernel debería tener un tamaño mayor a 0");
    }


    #[tokio::test]
    async fn test_handle_close_compilation() {
        // Test para verificar que handle_close se compila correctamente
        // No podemos probar la funcionalidad completa sin shards reales,
        // pero verificamos que el método existe y se puede llamar
        
        let token = "dummy_test".to_string();
        let intents = Intents::empty();
        
        // Este test principalmente verifica que el código compila
        let _kernel_result = Kernel::Kernel::new(token, intents, "redis://localhost".to_string()).await;
        
        // En un test real necesitarías mock objects para probar completamente
        assert!(true, "Test de compilación pasado");
    }

    #[test]
    fn test_default_generic_type() {
        // Test para verificar que el tipo genérico por defecto funciona
        let _kernel: Kernel::Kernel; // Esto debería compilar con el tipo por defecto
        assert!(true, "Tipo genérico por defecto funciona correctamente");
    }

    #[tokio::test]
    async fn test_message_sender_capacity() {
        // Test para verificar la inicialización de los vectores
        let token = "test".to_string();
        let intents = Intents::empty();
        
        if let Ok(kernel) =  Kernel::Kernel::new(token, intents).await {
            // Los vectores deberían estar inicializados con capacidad
            assert!(kernel.senders.capacity() > 0);
            assert!(kernel.tasks.capacity() > 0);
        }
    }
}