import { useState, useEffect } from "react";
import "./App.css";
import { Store } from '@tauri-apps/plugin-store';
import { enable, isEnabled } from '@tauri-apps/plugin-autostart';

interface Team {
  id: string;
  name: string;
  slug: string;
  planType: string;
  role: string;
  memberCount: number;
  fileCount: number;
  customerCount: number;
}

function App() {
  const [syncFolder, setSyncFolder] = useState<string>("");
  const [email, setEmail] = useState<string>("daniel.olsson@industrinat.se");
  const [password, setPassword] = useState<string>("");
  const [isLoggedIn, setIsLoggedIn] = useState<boolean>(false);
  const [apiUrl] = useState("https://flowen.eu");
  const [status, setStatus] = useState("Disconnected");
  
  // Team state
  const [teams, setTeams] = useState<Team[]>([]);
  const [selectedTeamId, setSelectedTeamId] = useState<string>("");
  const [loadingTeams, setLoadingTeams] = useState(false);

  // Hämta default-mapp när appen startar
  useEffect(() => {
    async function loadDefaultFolder() {
      const { invoke } = await import("@tauri-apps/api/core");
      try {
        const folder = await invoke<string>("get_default_sync_folder_cmd");
        setSyncFolder(folder);
      } catch (error) {
        console.error("Failed to get default folder:", error);
      }
    }
    loadDefaultFolder();
  }, []);

  // Hämta teams när användaren loggar in
  const fetchTeams = async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    
    try {
      setLoadingTeams(true);
      const result = await invoke<string>('get_user_teams');
      const data = JSON.parse(result);
      
      console.log('📊 Teams data:', data);
      
      if (data.teams && Array.isArray(data.teams)) {
        setTeams(data.teams);
        
        // Ladda sparat teamId från config, annars välj första teamet
        const store = await Store.load('.credentials.dat');
        const savedTeamId = await store.get<string>('selectedTeamId');
        
        if (savedTeamId && data.teams.find((t: Team) => t.id === savedTeamId)) {
          setSelectedTeamId(savedTeamId);
          // Sätt teamId i Rust state också
          await invoke('set_selected_team', { teamId: savedTeamId });
        } else if (data.teams.length > 0) {
          const firstTeamId = data.teams[0].id;
          setSelectedTeamId(firstTeamId);
          // Sätt teamId i Rust state
          await invoke('set_selected_team', { teamId: firstTeamId });
        }
      }
    } catch (error) {
      console.error('❌ Failed to fetch teams:', error);
      alert(`Failed to load teams: ${error}`);
    } finally {
      setLoadingTeams(false);
    }
  };

  // Spara valt team i config OCH Rust state
  const handleTeamChange = async (teamId: string) => {
    const { invoke } = await import("@tauri-apps/api/core");
    
    setSelectedTeamId(teamId);
    
    try {
      // Spara i config
      const store = await Store.load('.credentials.dat');
      await store.set('selectedTeamId', teamId);
      await store.save();
      console.log('💾 Selected team saved to config:', teamId);
      
      // Sätt i Rust state
      await invoke('set_selected_team', { teamId });
      console.log('✅ Selected team set in Rust state:', teamId);
      
    } catch (error) {
      console.error('Failed to save team selection:', error);
    }
  };
  // Auto-login vid start
useEffect(() => {
  async function tryAutoLogin() {
    // ✨ AKTIVERA AUTOSTART FÖRST
    try {
      const isAutostartEnabled = await isEnabled();
      if (!isAutostartEnabled) {
        await enable();
        console.log('✅ Autostart enabled');
      }
    } catch (error) {
      console.log('⚠️ Could not enable autostart:', error);
    }
      try {
        const store = await Store.load('.credentials.dat');
        const savedEmail = await store.get<string>('email');
        const savedPassword = await store.get<string>('password');
        
        if (savedEmail && savedPassword) {
          console.log('🔐 Found saved credentials, auto-logging in...');
          setEmail(savedEmail);
          setPassword(savedPassword);
          
          // Auto-login
          const { invoke } = await import("@tauri-apps/api/core");
          await invoke<string>("login_to_flowen", {
            email: savedEmail,
            password: savedPassword
          });
          
          console.log("✅ Auto-login successful");
          setIsLoggedIn(true);
          setStatus("Logged in");
          
          // Hämta teams efter login
          await fetchTeams();
          
          // Auto-start sync körs automatiskt via useEffect när selectedTeamId sätts
        }
      } catch (error) {
        console.log("No saved credentials or auto-login failed:", error);
      }
    }
    
    tryAutoLogin();
  }, []);

  // ✨ NY: Auto-start sync när selectedTeamId sätts (efter auto-login)
  useEffect(() => {
    let hasAutoStarted = false;
    
    async function autoStartSync() {
      // Kör bara om vi har ett selectedTeamId OCH är inloggad OCH inte redan kört
      if (selectedTeamId && isLoggedIn && !hasAutoStarted) {
        hasAutoStarted = true;
        console.log('🚀 Auto-starting sync for team:', selectedTeamId);
        
        // Liten delay för att säkerställa att allt är klart
        setTimeout(async () => {
          await handleStartSync();
        }, 500);
      }
    }
    
    autoStartSync();
  }, [selectedTeamId, isLoggedIn]);

  const handleLogin = async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    
    try {
      setStatus("Logging in...");
      
      await invoke<string>("login_to_flowen", {
        email: email,
        password: password
      });
      
      console.log("✅ Login successful");
      setIsLoggedIn(true);
      setStatus("Logged in");
      
      // Spara credentials för nästa gång
      const store = await Store.load('.credentials.dat');
      await store.set('email', email);
      await store.set('password', password);
      await store.save();
      console.log('💾 Credentials saved');
      
      // Hämta teams efter login
      await fetchTeams();
      
    } catch (error) {
      console.error("❌ Login error:", error);
      alert(`Login failed: ${error}`);
      setStatus("Login failed");
      setIsLoggedIn(false);
    }
  };

  const handleStartSync = async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    
    if (!isLoggedIn) {
      alert("Please login first!");
      return;
    }
    
    if (!selectedTeamId) {
      alert("Please select a team to sync!");
      return;
    }
    
    try {
      setStatus("Starting...");
      
      // 1. Skapa mappen
      const createResult = await invoke<string>("create_sync_folder", { 
        path: syncFolder 
      });
      console.log("✅ Create:", createResult);
      
      // 2. Montera som E:\
      try {
        const mountResult = await invoke<string>("mount_as_drive", {
          folderPath: syncFolder,
          driveLetter: "E"
        });
        console.log("✅ Mount:", mountResult);
      } catch (mountError) {
        console.warn("⚠️ Mount failed (OK to continue):", mountError);
      }
      
      // 3. ✨ NY: Initial sync FÖRST (ladda upp befintliga filer)
      setStatus("Scanning local files...");
      console.log("🔄 Starting initial sync...");
      
      try {
        const initialSyncResult = await invoke<string>("initial_sync", {
          folderPath: syncFolder
        });
        console.log("✅ Initial sync:", initialSyncResult);
      } catch (syncError) {
        console.warn("⚠️ Initial sync had issues:", syncError);
        // Fortsätt ändå - file watcher kan fortsätta fungera
      }
      
      // 4. Starta file watcher (lyssna på framtida ändringar)
      setStatus("Starting file watcher...");
      const watchResult = await invoke<string>("start_watching", {
        folderPath: syncFolder
      });
      console.log("✅ Watcher:", watchResult);
      
      const selectedTeam = teams.find(t => t.id === selectedTeamId);
      setStatus(`Syncing ${selectedTeam?.name || 'team'}...`);
      
    } catch (error) {
      console.error("❌ Error:", error);
      alert(`Failed: ${error}`);
      setStatus("Disconnected");
    }
  };

  const getSelectedTeamName = () => {
    const team = teams.find(t => t.id === selectedTeamId);
    return team ? `${team.name} (${team.fileCount} files)` : 'Select team...';
  };

  return (
    <div className="container">
      <h1>🚀 Flowen Sync</h1>
      
      <div className="card">
        <h2>🔐 Login</h2>
        
        <div className="setting">
          <label>Email:</label>
          <input 
            type="email" 
            value={email}
            onChange={(e) => setEmail(e.target.value)}
            placeholder="your@email.com"
            disabled={isLoggedIn}
          />
        </div>

        <div className="setting">
          <label>Password:</label>
          <input 
            type="password" 
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            placeholder="Your password"
            disabled={isLoggedIn}
          />
        </div>

        <button 
          className="primary" 
          onClick={handleLogin}
          disabled={isLoggedIn}
          style={{ marginBottom: "20px" }}
        >
          {isLoggedIn ? "✅ Logged In" : "Login to Flowen"}
        </button>
      </div>

      {isLoggedIn && (
        <div className="card">
          <h2>📁 Team Selection</h2>
          
          <div className="setting">
            <label>Select Team to Sync:</label>
            {loadingTeams ? (
              <div style={{ padding: '10px', color: '#94a3b8' }}>
                Loading teams...
              </div>
            ) : teams.length === 0 ? (
              <div style={{ padding: '10px', color: '#ef4444' }}>
                No teams found. Please create a team first.
              </div>
            ) : (
              <select 
                value={selectedTeamId}
                onChange={(e) => handleTeamChange(e.target.value)}
                style={{
                  width: '100%',
                  padding: '10px',
                  border: '1px solid rgba(255, 255, 255, 0.1)',
                  borderRadius: '6px',
                  background: 'rgba(0, 0, 0, 0.2)',
                  color: 'white',
                  fontSize: '14px',
                  cursor: 'pointer'
                }}
              >
                {teams.map(team => (
                  <option key={team.id} value={team.id}>
                    {team.name} ({team.fileCount} files, {team.memberCount} members) - {team.planType}
                  </option>
                ))}
              </select>
            )}
            
            {selectedTeamId && (
              <div style={{ 
                marginTop: '10px', 
                padding: '8px 12px', 
                background: 'rgba(14, 165, 233, 0.1)',
                border: '1px solid #0ea5e9',
                borderRadius: '6px',
                fontSize: '13px',
                color: '#0ea5e9'
              }}>
                ✓ Will sync with: {getSelectedTeamName()}
              </div>
            )}
          </div>
        </div>
      )}

      <div className="card">
        <h2>⚙️ Settings</h2>
        
        <div className="setting">
          <label>Sync Folder:</label>
          <input 
            type="text" 
            value={syncFolder}
            placeholder="C:\Users\Daniel\Flowen"
            readOnly
          />
        </div>

        <div className="setting">
          <label>Flowen Server:</label>
          <input 
            type="text" 
            value={apiUrl}
            readOnly
          />
        </div>

        <div className="setting">
          <label>Status:</label>
          <div style={{ display: 'flex', alignItems: 'center', gap: '10px' }}>
            <span className={`status ${status.toLowerCase().replace(/\s/g, '-')}`}>
              {status}
            </span>
            {status.startsWith("Syncing") && (
              <span style={{ 
                fontSize: '24px', 
                animation: 'spin 2s linear infinite',
                display: 'inline-block'
              }}>
                🔄
              </span>
            )}
          </div>
        </div>
      </div>

      <div className="actions">
        <button 
          className="primary" 
          onClick={handleStartSync}
          disabled={!isLoggedIn || !selectedTeamId || loadingTeams}
        >
          {!selectedTeamId ? "Select Team First" : "Start Syncing"}
        </button>
      </div>
    </div>
  );
}

export default App;
