# 🚀 Lancement Complet Dashboard Professionnel Aequitas

echo "🌐 DÉMARRAGE DU DASHBOARD PROFESSIONNEL AEQUITAS"
echo "========================================================"

echo "📊 Vérification de l'environnement..."
cd aequitas

# Vérifier que le dashboard est configuré
if [ ! -d "dashboard" ]; then
    echo "❌ Configuration du dashboard requise"
    echo "   Exécutez: ./scripts/setup_dashboard.sh"
    exit 1
fi

echo "✅ Configuration du dashboard détectée"

echo "🔧 Démarrage des services..."

# Configuration de l'environnement
export NODE_RPC_URL="http://localhost:23420"
export MINING_RPC_URL="http://localhost:23421" 
export API_PORT="3001"
export WS_PORT="3002"
export FRONTEND_PORT="3001"

# Vérifier si Docker est disponible
if command -v docker-compose >/dev/null 2>&1; then
    echo "🐳 Lancement avec Docker Compose..."
    
    # Lancement du stack monitoring
    cd dashboard
    docker-compose up -d
    
    echo ""
    echo "✅ Services Docker démarrés :"
    echo "   📊 Prometheus : http://localhost:9090"
    echo "   📈 Grafana : http://localhost:3000 (admin/admin123)"
    echo "   🔍 Node Exporter : http://localhost:9100"
    
    # Attendre quelques secondes que les services démarrent
    echo "⏳ Attente du démarrage des services..."
    sleep 10
    
    # Vérification que les services sont bien en cours d'exécution
    if docker-compose ps | grep -q "Up"; then
        echo "✅ Stack monitoring actif et fonctionnel !"
    else
        echo "⚠️ Certains services n'ont pas démarré"
        docker-compose ps
    fi
    
else
    echo "🚀 Lancement en mode développement..."
    
    # Lancement du backend API
    echo "🔧 Démarrage du Backend API..."
    cd dashboard/backend
    npm install > /dev/null 2>&1
    
    if command -v nodemon >/dev/null 2>&1; then
        echo "🔄 Backend démarré avec nodemon (rechargement auto)..."
        npm run dev > /dev/null 2>&1 &
    else
        echo "⚡ Backend démarré en mode production..."
        npm start > /dev/null 2>&1 &
    fi
    
    BACKEND_PID=$!
    
    # Lancement du frontend
    echo "🎨 Démarrage du Frontend React..."
    cd ../frontend
    npm install > /dev/null 2>&1
    
    npm run dev > /dev/null 2>&1 &
    FRONTEND_PID=$!
    
    # Attendre le démarrage
    sleep 5
    
    echo ""
    echo "✅ Services développement actifs :"
    echo "   🚀 Backend API : http://localhost:$API_PORT"
    echo "   🎨 Frontend : http://localhost:3001"
    echo "   📊 Logs temps réel disponibles"
    
    # Monitoring des processus
    echo ""
    echo "📊 Monitoring de l'état des services..."
    echo "Pour arrêter : Ctrl+C"
    
    # Attendre que les processus terminent
    wait $BACKEND_PID $FRONTEND_PID
fi

echo ""
echo "🎯 DASHBOARD AEQUITAS LANCÉ !"
echo ""
echo "📋 Services accessibles :"
if command -v docker-compose >/dev/null 2>&1 && docker-compose ps | grep -q "Up"; then
    echo "   📊 Monitoring production :"
    echo "     📈 Grafana : http://localhost:3000"
    echo "     📊 Prometheus : http://localhost:9090"
    echo "   🚀 API Backend : http://localhost:$API_PORT"
    echo "   🎨 Frontend Web : http://localhost:$FRONTEND_PORT"
else
    echo "   🚀 API Backend : http://localhost:$API_PORT"
    echo "   🎨 Frontend Web : http://localhost:3001"
fi

echo ""
echo "📞 Documentation :"
echo "   📚 API Docs : http://localhost:$API_PORT/api-docs"
echo "   🔍 Monitoring : Consulter les logs de chaque service"

echo ""
echo "🎯 Prochaines étapes :"
echo "   1. Connecter vos nodes Aequitas au dashboard"
echo "   2. Monitorer le réseau en temps réel"
echo "   3. Analyser les performances de mining"
echo "   4. Suivre la redistribution de solidarité"
echo "   5. Partager les stats sur les réseaux sociaux"

echo ""
echo "✨ Dashboard professionnel prêt pour l'écosystème Aequitas !"
echo "🚀 Début du monitoring complet..."